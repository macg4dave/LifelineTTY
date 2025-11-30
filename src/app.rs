use crate::{
    cli::RunOptions,
    config::{Config, DEFAULT_BAUD, DEFAULT_COLS, DEFAULT_DEVICE, DEFAULT_ROWS},
    lcd::Lcd,
    payload::Defaults as PayloadDefaults,
    payload::RenderFrame,
    serial::SerialPort,
    Error, Result,
};
use std::io::BufRead;
use std::time::{Duration, Instant};

/// Config for the daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub device: String,
    pub baud: u32,
    pub cols: u8,
    pub rows: u8,
    pub scroll_speed_ms: u64,
    pub page_timeout_ms: u64,
    pub button_gpio_pin: Option<u8>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            device: DEFAULT_DEVICE.to_string(),
            baud: DEFAULT_BAUD,
            cols: DEFAULT_COLS,
            rows: DEFAULT_ROWS,
            scroll_speed_ms: crate::payload::DEFAULT_SCROLL_MS,
            page_timeout_ms: crate::payload::DEFAULT_PAGE_TIMEOUT_MS,
            button_gpio_pin: None,
        }
    }
}

pub struct App {
    config: AppConfig,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn from_options(opts: RunOptions) -> Result<Self> {
        let cfg_file = Config::load_or_default()?;
        let merged = AppConfig::from_sources(cfg_file, opts);
        Ok(Self::new(merged))
    }

    /// Entry point for the daemon. Wire up serial + LCD here.
    pub fn run(&self) -> Result<()> {
        let mut port = SerialPort::connect(&self.config.device, self.config.baud)?;
        let mut lcd = Lcd::new(self.config.cols, self.config.rows);

        lcd.render_boot_message()?;
        port.send_line("INIT")?;

        let mut state = crate::state::RenderState::new(Some(PayloadDefaults {
            scroll_speed_ms: self.config.scroll_speed_ms,
            page_timeout_ms: self.config.page_timeout_ms,
        }));
        let mut buffer = String::new();
        let mut last_render = Instant::now();
        let min_render_interval = Duration::from_millis(200);
        let mut current_frame: Option<RenderFrame> = None;
        let mut next_page = Instant::now();
        let mut next_scroll = Instant::now();
        let mut scroll_offsets = (0usize, 0usize);
        let mut button = Button::new(self.config.button_gpio_pin).ok();

        loop {
            let now = Instant::now();

            if let Some(btn) = button.as_mut() {
                if btn.is_pressed() {
                    if let Some(frame) = state.next_page() {
                        current_frame = Some(frame);
                        scroll_offsets = (0, 0);
                        next_scroll =
                            now + Duration::from_millis(self.config.scroll_speed_ms);
                        if let Some(frame) = current_frame.as_ref() {
                            next_page = now + Duration::from_millis(frame.page_timeout_ms);
                            render_if_allowed(
                                &mut lcd,
                                frame,
                                &mut last_render,
                                min_render_interval,
                                scroll_offsets,
                            )?;
                        }
                    }
                }
            }

            buffer.clear();
            let read = port.read_line(&mut buffer)?;
            if read > 0 {
                let line = buffer.trim_end_matches(&['\r', '\n'][..]).trim();
                if !line.is_empty() {
                    match state.ingest(line) {
                        Ok(Some(frame)) => {
                            current_frame = Some(frame);
                            scroll_offsets = (0, 0);
                            next_scroll =
                                now + Duration::from_millis(self.config.scroll_speed_ms);
                            if let Some(frame) = current_frame.as_ref() {
                                next_page =
                                    now + Duration::from_millis(frame.page_timeout_ms);
                                render_if_allowed(
                                    &mut lcd,
                                    frame,
                                    &mut last_render,
                                    min_render_interval,
                                    scroll_offsets,
                                )?;
                            }
                        }
                        Ok(None) => { /* duplicate */ }
                        Err(err) => eprintln!("frame error: {err}"),
                    }
                }
            }

            if state.len() > 1 && now >= next_page {
                if let Some(frame) = state.next_page() {
                    current_frame = Some(frame);
                    scroll_offsets = (0, 0);
                    if let Some(frame) = current_frame.as_ref() {
                        next_page = now + Duration::from_millis(frame.page_timeout_ms);
                        render_if_allowed(
                            &mut lcd,
                            frame,
                            &mut last_render,
                            min_render_interval,
                            scroll_offsets,
                        )?;
                    }
                }
            }

            if let Some(frame) = current_frame.as_ref() {
                let needs_scroll = line_needs_scroll(&frame.line1, lcd.cols() as usize)
                    || line_needs_scroll(&frame.line2, lcd.cols() as usize);
                if needs_scroll && now >= next_scroll {
                    scroll_offsets = (
                        advance_offset(&frame.line1, lcd.cols() as usize, scroll_offsets.0),
                        advance_offset(&frame.line2, lcd.cols() as usize, scroll_offsets.1),
                    );
                    next_scroll =
                        now + Duration::from_millis(frame.scroll_speed_ms);
                    render_if_allowed(
                        &mut lcd,
                        frame,
                        &mut last_render,
                        min_render_interval,
                        scroll_offsets,
                    )?;
                }
            }
        }
    }
}

impl AppConfig {
    pub fn from_sources(config: Config, opts: RunOptions) -> Self {
        Self {
            device: opts
                .device
                .unwrap_or_else(|| config.device.clone()),
            baud: opts.baud.unwrap_or(config.baud),
            cols: opts.cols.unwrap_or(config.cols),
            rows: opts.rows.unwrap_or(config.rows),
            scroll_speed_ms: config.scroll_speed_ms,
            page_timeout_ms: config.page_timeout_ms,
            button_gpio_pin: config.button_gpio_pin,
        }
    }
}

fn render_frame(lcd: &mut Lcd, frame: &RenderFrame) -> Result<()> {
    render_frame_with_scroll(lcd, frame, (0, 0))
}

fn render_bar(percent: u8, width: usize) -> String {
    let filled = (percent as usize * width) / 100;
    let mut s = String::with_capacity(width);
    for i in 0..width {
        s.push(if i < filled { '#' } else { ' ' });
    }
    s
}

fn render_if_allowed(
    lcd: &mut Lcd,
    frame: &RenderFrame,
    last_render: &mut Instant,
    min_interval: Duration,
    scroll_offsets: (usize, usize),
) -> Result<()> {
    let now = Instant::now();
    if now.duration_since(*last_render) < min_interval {
        return Ok(());
    }
    *last_render = now;
    render_frame_with_scroll(lcd, frame, scroll_offsets)
}

fn render_frame_with_scroll(
    lcd: &mut Lcd,
    frame: &RenderFrame,
    offsets: (usize, usize),
) -> Result<()> {
    if frame.clear {
        lcd.render_boot_message()?;
    }

    let width = lcd.cols() as usize;
    let line1 = view_with_scroll(&frame.line1, width, offsets.0);
    let line2 = if let Some(percent) = frame.bar_percent {
        render_bar(percent, width)
    } else {
        view_with_scroll(&frame.line2, width, offsets.1)
    };

    lcd.write_line(0, &line1)?;
    lcd.write_line(1, &line2)?;
    Ok(())
}

fn line_needs_scroll(text: &str, width: usize) -> bool {
    text.chars().count() > width
}

fn advance_offset(text: &str, width: usize, current: usize) -> usize {
    let len = text.chars().count();
    if len <= width {
        return 0;
    }
    let cycle = len + 1; // include a space gap
    (current + 1) % cycle
}

fn view_with_scroll(text: &str, width: usize, offset: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= width {
        return text.to_string();
    }
    let gap = vec![' '];
    let mut cycle: Vec<char> = chars.clone();
    cycle.extend_from_slice(&gap);
    cycle.extend_from_slice(&chars);

    let start = offset.min(cycle.len().saturating_sub(1));
    cycle
        .iter()
        .cycle()
        .skip(start)
        .take(width)
        .collect()
}

#[cfg(target_os = "linux")]
struct Button {
    pin: rppal::gpio::InputPin,
    last: Instant,
    debounce: Duration,
}

#[cfg(target_os = "linux")]
impl Button {
    fn new(pin: Option<u8>) -> Result<Self> {
        let pin = match pin {
            Some(p) => p,
            None => return Err(Error::InvalidArgs("no button pin configured".into())),
        };
        let gpio = rppal::gpio::Gpio::new()
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        let input = gpio
            .get(pin)
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            .into_input_pullup();
        Ok(Self {
            pin: input,
            last: Instant::now(),
            debounce: Duration::from_millis(150),
        })
    }

    fn is_pressed(&mut self) -> bool {
        let now = Instant::now();
        if self.pin.is_low() && now.duration_since(self.last) > self.debounce {
            self.last = now;
            true
        } else {
            false
        }
    }
}

#[cfg(not(target_os = "linux"))]
struct Button;

#[cfg(not(target_os = "linux"))]
impl Button {
    fn new(_pin: Option<u8>) -> Result<Self> {
        Err(Error::InvalidArgs("button unsupported on this platform".into()))
    }

    fn is_pressed(&mut self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn config_from_options() {
        let opts = RunOptions {
            device: Some("/dev/ttyUSB1".into()),
            baud: Some(57_600),
            cols: Some(16),
            rows: Some(2),
        };
        let cfg = AppConfig::from_sources(Config::default(), opts.clone());
        assert_eq!(cfg.device, "/dev/ttyUSB1");
        assert_eq!(cfg.baud, 57_600);
        assert_eq!(cfg.cols, 16);
        assert_eq!(cfg.rows, 2);

        let app = App::from_options(opts).unwrap();
        assert_eq!(app.config.device, "/dev/ttyUSB1");
    }

    #[test]
    fn config_prefers_file_values_when_cli_missing() {
        let cfg_file = Config {
            device: "/dev/ttyS0".into(),
            baud: 9_600,
            cols: 16,
            rows: 2,
            scroll_speed_ms: crate::config::DEFAULT_SCROLL_MS,
            page_timeout_ms: crate::config::DEFAULT_PAGE_TIMEOUT_MS,
            button_gpio_pin: None,
        };
        let opts = RunOptions::default();
        let merged = AppConfig::from_sources(cfg_file.clone(), opts);
        assert_eq!(merged.device, cfg_file.device);
        assert_eq!(merged.baud, cfg_file.baud);
        assert_eq!(merged.cols, cfg_file.cols);
        assert_eq!(merged.rows, cfg_file.rows);
    }
}
