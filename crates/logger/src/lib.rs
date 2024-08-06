use chrono::{DateTime, Local};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use nu_ansi_term::{Color, Style};
use tracing::{error, Level};
use tracing_subscriber::Layer;

/// Custom Logging layer for [tracing](https://github.com/tokio-rs/tracing)
pub struct LoggingLayer {
    pub logfile: Option<String>,
}

impl<S> Layer<S> for LoggingLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut fields = HashMap::new();
        let mut visitor = LogVisitor(&mut fields);
        event.record(&mut visitor);

        let binding = fields.clone();
        let message = binding.get("message");
        let target = binding.get("target");

        fields.remove("message");
        fields.remove("target");

        let mut logfmt = LogFormat {
            timestamp: Local::now(),
            message,
            target,
            level: event.metadata().level(),
            fields: fields.clone(),
        };

        if let Some(logfile) = &self.logfile {
            let path = Path::new(logfile);
            if !path.exists() {
                File::create(logfile).unwrap();
            }
            let mut file = OpenOptions::new()
                .append(true)
                .open(path)
                .map_err(|err| error!("Cannot open log file: {err}"))
                .unwrap();
            if let Err(e) = writeln!(file, "{}", logfmt.without_ansi()) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }

        match event.metadata().level() {
            &Level::ERROR => {
                eprintln!("{}", logfmt.with_ansi())
            }
            _ => {
                println!("{}", logfmt.with_ansi())
            }
        };
    }
}

/// Formatting Logs
struct LogFormat<'a> {
    timestamp: DateTime<Local>,
    message: Option<&'a String>,
    level: &'a Level,
    target: Option<&'a String>,
    fields: HashMap<String, String>,
}
impl<'a> LogFormat<'a> {
    /// Formatting logs with styling & colors
    pub fn with_ansi(&mut self) -> String {
        format!(
            "{}{:5}{} {} {}",
            self.format_timestamp(),
            self.format_level(),
            self.format_target(),
            self.format_message(),
            self.format_fields()
        )
    }
    /// Formatting logs without styling or colors to write it to a log file
    pub fn without_ansi(&self) -> String {
        format!(
            "{} {:<6}[{}]{} {}",
            self.timestamp,
            self.level,
            self.target.unwrap_or(&"".to_string()),
            self.message.unwrap_or(&"".to_string()),
            self.fields_as_str()
        )
    }

    /// Custom format for timestamp
    pub fn format_timestamp(&mut self) -> String {
        format!(
            "{} ",
            Style::new().dimmed().paint(self.timestamp.to_rfc3339())
        )
    }
    /// Custom format for log level
    pub fn format_level(&self) -> String {
        format!(
            " {} ",
            match *self.level {
                Level::TRACE => Color::Purple,
                Level::DEBUG => Color::Blue,
                Level::INFO => Color::Green,
                Level::WARN => Color::Yellow,
                Level::ERROR => Color::Red,
            }
            .paint(self.level.as_str())
        )
    }
    /// Custom format for target
    fn format_target(&self) -> String {
        if let Some(target) = self.target {
            format!("[{}]", Style::new().italic().dimmed().paint(target))
        } else {
            "".to_string()
        }
    }
    /// Custom format for log message
    pub fn format_message(&self) -> String {
        Style::new()
            .bold()
            .paint(self.message.unwrap_or(&"".to_string()))
            .to_string()
    }
    /// Custom format for fields
    pub fn format_fields(&self) -> String {
        Style::new()
            .dimmed()
            .paint(self.fields_as_str())
            .to_string()
    }

    /// combaining fields into string
    fn fields_as_str(&self) -> String {
        let mut fields_str = String::new();
        for (key, val) in self.fields.iter() {
            fields_str.push_str(&format!("{key}: {val} "));
        }

        fields_str
    }
}

struct LogVisitor<'a>(&'a mut HashMap<String, String>);

impl<'a> tracing::field::Visit for LogVisitor<'a> {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{value:?}"));
    }
}
