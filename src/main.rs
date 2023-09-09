use maria::{handler, Arc, HandlerFn, Mutex, Request, Response, Router};
use std::path::Path;
use std::path::PathBuf;
use std::{fs, io};

fn list_dir_entries(path: &Path) -> Result<Vec<(PathBuf)>, std::io::Error> {
    let entries = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    return Ok(entries);
}
fn render_dir_entries(entries: Vec<PathBuf>) -> Vec<String> {
    let mut list = Vec::new();
    for entry in entries.iter() {
        let link = format!("{}", entry.as_path().to_str().unwrap());
        let text = format!("<a href=/file/{}>{}<a>", link, entry.to_str().unwrap());
        list.push(text);
    }
    return list;
}

#[tokio::main]
async fn main() {
    let mut router = Router::new();

    let home: HandlerFn = handler!(_req, res, {
        res.send_html("QWE");
    });

    router.get("/", vec![home]);

    let file_handler: HandlerFn = handler!(req, res, {
        println!("{:?}", req.params);
        match req.params.get("*fpath") {
            // check if there is a path
            Some(path) => {
                // get content metadata
                let metadata = std::fs::metadata(path);
                match metadata {
                    // check content type
                    Ok(metadata) => {
                        if metadata.is_dir() {
                            match list_dir_entries(Path::new(path)) {
                                Ok(entries) => {
                                    let elements_list = render_dir_entries(entries);
                                    println!("{:?}", elements_list);

                                    let mut response_string = String::new();
                                    for element in elements_list.iter() {
                                        response_string += &format!("{}\n<br>", element);
                                    }

                                    res.send_html(response_string.as_str());
                                }
                                Err(e) => res.send_html(format!("Content error: {:?}", e).as_str()),
                            }
                        } else if metadata.is_file() {
                            res.send_html("Content is a file.");
                        }
                        // else, content return not found
                        else {
                            res.send_html(format!("Content not found in path").as_str())
                        }
                    }
                    Err(e) => res.send_html(format!("Content error: {:?}", e).as_str()),
                }
            }
            None => res.send_html(format!("Content not found in path").as_str()),
        }
    });

    router.get("/file/:*fpath", vec![file_handler]);

    println!("listenign 8080!");
    router.listen(8080).await;
}
