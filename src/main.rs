use reqwest::blocking::get;
use select::document::Document;
use select::node::Node;
use select::predicate::{Name, Attr};
use select::predicate::Predicate;
use url::Url;
use minifier::css::minify;

fn main() {
    // Specify the URL of the webpage you want to scrape
    let url = "https://rust-trends.com/newsletter/rust-in-action-10-project-ideas-to-elevate-your-skills/";

    // Send an HTTP GET request to the specified URL
    let response = reqwest::blocking::get(url).expect("Failed to send request");

    // Check if the request was successful (status code 200)
    if response.status().is_success() {
        // Read the HTML content of the response
        let html = response.text().expect("Failed to read response");

        // Parse the HTML content using the select library
        let document = Document::from_read(html.as_bytes()).expect("Failed to parse HTML");

        // Extract and fetch linked CSS files
        fetch_css_files(&document);

        // Extract and save image files
        fetch_and_save_images(&document);
        fetch_and_save_js_files(&document);

        // Save the HTML content to a file
        save_html_to_file("output.html", html);
    } else {
        println!("Failed to fetch the webpage. Status code: {}", response.status());
    }
}




fn fetch_and_save_js_files(document: &Document) {
    // Find all <script> elements and extract the "src" attribute
    let js_links = document
        .find(Name("script"))
        .filter_map(|node| node.attr("src"));

    // Fetch and save each JS file
    for link in js_links {
        fetch_and_save_js(link);
    }
}

fn fetch_and_save_js(js_url: &str) {
    // Send an HTTP GET request to the specified JS URL
    let response = reqwest::blocking::get(js_url);

    // Check if the request was successful (status code 200)
    if let Ok(response) = response {
        if response.status().is_success() {
            // Read the JS content of the response
            let js_content = response.text().expect("Failed to read JS response");

            // Save the JS content to a file
            let file_name = format!("script{}", get_file_name_from_url(js_url));
            std::fs::write(&file_name, js_content).expect("Failed to save JS file");

            println!("JS file saved: {}", file_name);
        } else {
            println!("Failed to fetch JS file. Status code: {}", response.status());
        }
    } else {
        println!("Failed to send request for JS file.");
    }
}

fn fetch_css_files(document: &Document) {
    // Find all <link> elements with the "stylesheet" rel attribute
    let css_links = document
        .find(Name("link").and(Attr("rel", "stylesheet")))
        .filter_map(|node| node.attr("href"));

    // Fetch each linked CSS file
    for link in css_links {
        fetch_and_save_css(link);
    }

    // Extract and save embedded CSS in style tags
    let embedded_css = document
        .find(Name("style"))
        .filter_map(|node| Some(node.text()))
        .collect::<Vec<_>>()
        .join("\n");

    if !embedded_css.is_empty() {
        let minified_css = minify(&embedded_css);
        save_css_to_file("embedded_css.css", minified_css);
        println!("Embedded CSS saved: embedded_css.css");
    }
}

fn fetch_and_save_css(css_url: &str) {
    // Send an HTTP GET request to the specified CSS URL
    let response = get(css_url);

    // Check if the request was successful (status code 200)
    if let Ok(response) = response {
        if response.status().is_success() {
            // Read the CSS content of the response
            let css_content = response.text().expect("Failed to read CSS response");

            // Minify the CSS content
            let minified_css = minify(&css_content);

            // Save the minified CSS content to a file
            let file_name = format!("style{}", get_file_name_from_url(css_url));
            save_css_to_file(&file_name, minified_css);

            println!("Linked CSS file saved: {}", file_name);
        } else {
            println!("Failed to fetch CSS file. Status code: {}", response.status());
        }
    } else {
        println!("Failed to send request for CSS file.");
    }
}

fn get_file_name_from_url(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|u| u.path_segments().map(|segments| segments.last().map(|s| s.to_string())))
        .flatten()
        .unwrap_or_else(|| String::from(""))
}

fn save_css_to_file(file_path: &str, css_content: Result<minifier::css::Minified<'_>, &str>) {
    match css_content {
        Ok(minified) => {
            std::fs::write(file_path, minified.to_string()).expect("Failed to save CSS file");
        },
        Err(e) => {
            println!("Failed to minify CSS: {}", e);
        }
    }
}

fn save_html_to_file(file_path: &str, html_content: String) {
    std::fs::write(file_path, html_content).expect("Failed to save HTML file");
    println!("HTML content saved: {}", file_path);
}

fn fetch_and_save_images(document: &Document) {
    // Find all <img> elements and extract the "src" attribute
    let image_links = document
        .find(Name("img"))
        .filter_map(|node| node.attr("src"));

    // Fetch and save each image
    for link in image_links {
        fetch_and_save_image("https://rust-trends.com/newsletter/rust-in-action-10-project-ideas-to-elevate-your-skills/", link);
    }
}

fn fetch_and_save_image(base_url: &str, image_url: &str) {
    // Convert relative URL to absolute URL using the base URL
    let absolute_url = if image_url.starts_with("http") {
        image_url.to_string() // Already an absolute URL
    } else {
        let base = Url::parse(base_url).expect("Failed to parse base URL");
        let absolute = base.join(image_url).expect("Failed to create absolute URL");
        absolute.to_string()
    };

    // Send an HTTP GET request to the specified image URL
    let response = get(&absolute_url);

    // Check if the request was successful (status code 200)
    if let Ok(response) = response {
        if response.status().is_success() {
            // Read the image content of the response
            let image_content = response.bytes().expect("Failed to read image response");

            // Save the image content to a file
            let file_name = format!("image{}", get_file_name_from_url(&absolute_url));
            std::fs::write(&file_name, image_content).expect("Failed to save image file");

            println!("Image file saved: {}", file_name);
        } else {
            println!("Failed to fetch image file. Status code: {}", response.status());
        }
    } else {
        // Print more details about the error
        println!("Failed to send request for image file. Error: {:?}", response.err());
    }
}