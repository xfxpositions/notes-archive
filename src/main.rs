use dotenv::dotenv;
use maria::{handler, Arc, HandlerFn, Mutex, Request, Response, Router};
use markdown;
use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsStr;
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::path::PathBuf;
use std::{fs, fs::File, io};

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

// also deletes everything
fn write_string_to_file_truncate(fpath: &Path, content: &String) -> Result<(), std::io::Error> {
    let fpath = std::path::Path::new("./test.txt");
    let mut file = fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .open(fpath)?;

    let _ = file.write(content.as_bytes())?;
    return Ok(());
}

#[tokio::main]
async fn main() {
    //dotenv
    dotenv().ok();

    let _ = match env::var("fpassword") {
        Ok(fpassword) => fpassword,
        Err(e) => panic!(".env error {:?}", e),
    };

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

    let edit_file: HandlerFn = handler!(req, res, {
        let mut fpath = req.params.get("*fpath");
        let formatted_path = format!("{}/{}", &PUBLIC_DIR, fpath.unwrap());
        if !fpath.unwrap().contains(&PUBLIC_DIR) {
            fpath = Some(&formatted_path);
        }
        println!("{:?}", fpath);
        if fpath.is_none() {
            res.set_status_code_raw(401);
            res.send_html("err: please provide a file path");
            return;
        }
        // check fpath is exist
        match fs::metadata(fpath.unwrap()) {
            Ok(_) => {}
            Err(_) => {
                res.send_html("err: file not found");
                return;
            }
        }
        let mut render_str = format!("<script>const fpath = \"{}\"</script>", fpath.unwrap());
        render_str.push_str(
            format!(
                "<script>const fcontent = `{}`</script>",
                fs::read_to_string(fpath.unwrap()).unwrap()
            )
            .as_str(),
        );

        match std::fs::read_to_string("./src/views/edit.html") {
            Ok(mut html_str) => {
                html_str = render_str + html_str.as_str();
                res.send_html(html_str.as_str());
            }
            Err(e) => res.send_html(format!("Content error: {:?}", e).as_str()),
        }
    });

    #[derive(Deserialize, Serialize, Clone)]
    struct EditFilePost {
        content: String,
        password: String,
    }

    let edit_file_post: HandlerFn = handler!(req, res, {
        let body: Result<EditFilePost, serde_json::Error> = serde_json::from_str(req.body.as_str());
        if body.is_err() {
            res.set_status_code_raw(400);
            res.send_text(format!("File err: {:?}", body.err()).as_str());
            return;
        }

        let body_unwrap = body.unwrap().clone();

        // check password
        if body_unwrap.password != env::var("fpassword").unwrap() {
            res.set_status_code_raw(403);
            res.send_text(format!("wrong password maan",).as_str());
            return;
        }

        let fpath = req.params.get("*fpath");
        if fpath.is_none() {
            res.set_status_code_raw(401);
            res.send_html("err: please provide a file path");
            return;
        }
        //finally

        match write_string_to_file_truncate(Path::new(fpath.unwrap()), &body_unwrap.content) {
            Ok(_) => {
                res.set_status_code_raw(200);
                res.send_text(format!("Ok").as_str());
                return;
            }
            Err(e) => {
                res.set_status_code_raw(500);
                res.send_text(format!("some error happened, {:?}", e).as_str());
                return;
            }
        }
    });

    router.post("/edit/:*fpath", vec![edit_file_post]);

    router.get("/editz/:*fpath", vec![edit_file]);

    println!("listenign 8080!");
    router.listen(8080).await;
}
