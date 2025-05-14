use std::sync::OnceLock;
use tera::{Context, Tera};

static TEMPLATES: OnceLock<Tera> = OnceLock::new();

pub fn init_templates() -> Result<(), Box<dyn std::error::Error>> {
    let mut tera = Tera::default();

    let chat_style_css_content = include_str!("../templates/chat_style.css.tera");
    tera.add_raw_template("chat_style.css", chat_style_css_content)?;
    let css_injection_js_content = include_str!("../templates/css_injection.js.tera");
    tera.add_raw_template("css_injection.js", css_injection_js_content)?;

    TEMPLATES
        .set(tera)
        .map_err(|_| "Templates already initialized")?;
    Ok(())
}

fn get_templates() -> &'static Tera {
    TEMPLATES.get().expect("Templates not initialized")
}

pub fn generate_css_injection_script(
    author_font_import_url: &str,
    message_font_import_url: &str,
    background_color: &str,
    message_font_css_family: &str,
    author_color: &str,
    author_font_css_family: &str,
    message_color: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut context = Context::new();

    context.insert("author_font_import_url", author_font_import_url);
    context.insert("message_font_import_url", message_font_import_url);
    context.insert("background_color", background_color);
    context.insert("message_font_css_family", message_font_css_family);
    context.insert("author_color", author_color);
    context.insert("author_font_css_family", author_font_css_family);
    context.insert("message_color", message_color);

    let css_content = get_templates().render("chat_style.css", &context)?;

    let mut js_context = Context::new();
    js_context.insert("css_content", &css_content);

    // Render the JS template with the CSS content
    let js_content = get_templates().render("css_injection.js", &js_context)?;

    Ok(js_content)
}
