# templates/

This directory holds Tera templates used by the `cr8s` backend.

## 📧 Email Templates

Located in `templates/email/`, these templates are rendered and sent as HTML emails from within the application.

### Currently included:

- `digest.html` – Used for sending a summary or digest email to users

## 🔧 Development Notes

- Templates use the `.html` extension but are parsed by the Tera engine
- Variables inside templates follow Tera syntax: `{{ var_name }}`
- Templates are auto-reloaded in development (`cargo run`) for fast iteration
- On deployment, templates may be embedded or loaded from disk as configured via `Rocket.toml`

## Structure

