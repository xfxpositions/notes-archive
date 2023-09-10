use maria::{handler, Arc, HandlerFn, Mutex, Request, Response, Router};
use markdown;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::{fs, io};

fn list_dir_entries(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let entries = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    return Ok(entries);
}

const HEAD: &str = "<h2>Divine archive of Tengri</h2><hr/><h3>Contents</h3>";
const FOOTER: &str = "<hr><a href=\"https://yusufkaraca.dev\">yusufkaraca.dev   </a><a href=\"https://github.com/xfxpositions/divine-arhive\">source code</a>";
// static = ./static
// public/share/ = ./public/share
const PUBLIC_DIR: &str = "static";

// for url
// /file/doc.pdf
const PUBLIC_VIR_DIR: &str = "/file";

fn render_dir_entries(entries: Vec<PathBuf>) -> Vec<String> {
    let mut list = Vec::new();

    list.push(HEAD.to_string());

    let updir_1 = Path::new(entries[0].parent().unwrap());
    let updir_2 = updir_1.parent().unwrap();
    let updir_element = format!("<a href=/file/{}>{}<a>", updir_2.to_string_lossy(), "../");

    list.push(updir_element);
    for entry in entries.iter() {
        let link = format!("{}", entry.as_path().to_str().unwrap());
        let text = format!("<a href=/file/{}>{}<a>", link, entry.to_str().unwrap());
        list.push(text);
    }
    list.push(FOOTER.to_string());

    return list;
}

fn serve_file(fpath: &Path) -> Result<String, std::io::Error> {
    let content = fs::read_to_string(fpath)?;
    return Ok(content);
}

fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

fn render_markdown(content: &String) -> String {
    return markdown::to_html(content);
}

#[tokio::main]
async fn main() {
    let mut router = Router::new();

    let home: HandlerFn = handler!(_req, res, {
        res.send_html("QWE");
    });

    router.get("/", vec![home]);

    let file_handler: HandlerFn = handler!(req, res, {
        match req.params.get("*fpath") {
            // check if there is a path
            Some(mut path) => {
                println!("original path: {}", path);
                //let mut path_parts: Vec<&str> = path.split("/").collect();
                let formatted_path = format!("{}/{}", &PUBLIC_DIR, path);
                if !path.contains(&PUBLIC_DIR) {
                    path = &formatted_path;
                }
                println!("path: {}", path);
                // get content metadata
                let metadata = std::fs::metadata(path);
                match metadata {
                    // check content type
                    Ok(metadata) => {
                        if metadata.is_dir() {
                            match list_dir_entries(Path::new(path)) {
                                Ok(entries) => {
                                    let elements_list = render_dir_entries(entries);

                                    let mut response_string = String::new();
                                    for element in elements_list.iter() {
                                        response_string += &format!("{}\n<br>", element);
                                    }

                                    res.send_html(response_string.as_str());
                                }
                                Err(e) => res.send_html(format!("Content error: {:?}", e).as_str()),
                            }
                        } else if metadata.is_file() {
                            match serve_file(Path::new(path)) {
                                Ok(content) => {
                                    let file_ext = get_extension_from_filename(path);
                                    if file_ext == Some("html") {
                                        res.send_html(content.as_str());
                                    } else if file_ext == Some("md") {
                                        let response_str = render_markdown(&content);
                                        res.send_html(response_str.as_str());
                                    } else {
                                        res.send_text(content.as_str());
                                    }
                                }

                                Err(e) => res.send_html(format!("Content error: {:?}", e).as_str()),
                            }
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

    let vir_path = format!("{}/:*fpath", PUBLIC_VIR_DIR);
    router.get(vir_path.as_str(), vec![file_handler]);

    let edit_file: HandlerFn = handler!(req, res, {});

    router.post("/edit/:*fpath", vec![edit_file]);

    println!("listenign 8080!");
    router.listen(8080).await;
}
