extern crate web_view;

use web_view::*;

fn main() {
    let size = (700, 400);
    let resizable = true;
    let debug = false;
    let titlebar_transparent = true;
    let frontend_cb = |_webview: &mut _, _arg: &_, _userdata: &mut _| {};
    let userdata = ();

    let html = format!(
        r#"
    <html>
        <head>
        <style>{css}</style>
        <link href="https://fonts.googleapis.com/css?family=Roboto:300,400,500|Material+Icons" rel="stylesheet">
        <link rel="stylesheet" href="https://unpkg.com/material-components-web-elm@9.1.0/dist/material-components-web-elm.min.css">
        <script src="https://unpkg.com/material-components-web-elm@9.1.0/dist/material-components-web-elm.min.js"></script>
        <script>{js}</script>
        </head>
        <body>
        <div id="app"></div>
        <script>
        var app = Elm.Main.init({{ node: document.getElementById('app') }});
        app.ports.sendMessage.subscribe(function(message) {{
            app.ports.messageReceiver.send(message);
        }});
        </script>
        </body>
    </html>
    "#,
        css = r#"body { background: #ffffff; }"#,
        js = include_str!("../www/main.js")
    );

    std::fs::write("index.html", html.clone()).unwrap();

    run(
        "",
        Content::Html(html),
        Some(size),
        resizable,
        debug,
        titlebar_transparent,
        move |mut webview| {
            webview.set_background_color(0.11, 0.12, 0.13, 1.0);
        },
        frontend_cb,
        userdata,
    );
}

// #![windows_subsystem = "windows"]

// extern crate web_view;

// use web_view::*;

// fn main() {
// 	let size = (800, 600);
// 	let resizable = true;
// 	let debug = true;
// 	let init_cb = |_webview| {};
// 	let frontend_cb = |_webview: &mut _, _arg: &_, _userdata: &mut _| {};
// 	let userdata = ();
// 	run(
// 		"Minimal webview example",
// 		Content::Url("https://en.m.wikipedia.org/wiki/Main_Page"),
// 		Some(size),
// 		resizable,
// 		debug,
//         false,
// 		init_cb,
// 		frontend_cb,
// 		userdata
// 	);
// }
