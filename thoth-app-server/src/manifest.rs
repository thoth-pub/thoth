//! Utility module to generate the manifest.json file

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn manifest_source() -> String {
    format!(
        r##"
{{
  "name": "Thoth",
  "version": "{VERSION}",
  "description": "Bibliographical metadata management system.",
  "display": "standalone",
  "scope": "/admin",
  "start_url": ".",
  "background_color": "#FFDD57",
  "theme_color": "#FFDD57",
  "icons": [
      {{
        "src": "https://cdn.thoth.pub/android-icon-36x36.png",
        "sizes": "36x36",
        "type": "image\/png",
        "density": "0.75"
      }},
      {{
        "src": "https://cdn.thoth.pub/android-icon-48x48.png",
        "sizes": "48x48",
        "type": "image\/png",
        "density": "1.0"
      }},
      {{
        "src": "https://cdn.thoth.pub/android-icon-72x72.png",
        "sizes": "72x72",
        "type": "image\/png",
        "density": "1.5"
      }},
      {{
        "src": "https://cdn.thoth.pub/android-icon-96x96.png",
        "sizes": "96x96",
        "type": "image\/png",
        "density": "2.0"
      }},
      {{
        "src": "https://cdn.thoth.pub/android-icon-144x144.png",
        "sizes": "144x144",
        "type": "image\/png",
        "density": "3.0"
      }},
      {{
        "src": "https://cdn.thoth.pub/android-icon-192x192.png",
        "sizes": "192x192",
        "type": "image\/png",
        "density": "4.0"
      }}
  ]
}}
"##
    )
}
