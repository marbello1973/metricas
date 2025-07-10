use chrono::Local;
use serde::{Deserialize, Serialize};
use crossterm::{
    ExecutableCommand, cursor, queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{self},
};
use std::{
    io::{Write, stdout},
    thread,
    time::{Duration},
};
use sysinfo::System;

// --------------------------
// Estructuras de Datos 
// --------------------------
struct SystemMetrics {
    cpu_usage: f32,
    mem_usage: f32,
    disk_usage: f32,
}

// --------------------------
// Implementaciones
// --------------------------
impl SystemMetrics {
    fn new() -> Self {
        SystemMetrics {
            cpu_usage: 0.0,
            mem_usage: 0.0,
            disk_usage: 0.0,
        }
    }

    fn update(&mut self, sys: &System) {
        self.cpu_usage = sys.global_cpu_usage();
        self.mem_usage = (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0;
        
        // Uso simplificado de disco (requiere más lógica para sistemas reales)
        self.disk_usage = 30.0; // Ejemplo estático
    }
}

// --------------------------
// Estructuras y Funciones Auxiliares
// --------------------------

/// Representa los datos del clima.
/// En una aplicación real, esto se obtendría de una API.
struct Weather {
    icon: String,
    temp: f32,
}


#[derive(Debug, Serialize, Deserialize)]
struct CurrentWeather {
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    current_weather: CurrentWeather,
}



impl Weather {
    fn new() -> Self {
        Weather {
            icon: "☀️".to_string(),
            temp: 0.0,
        }
    }
    
    /// Actualiza los datos del clima (actualmente con valores estáticos).
    async fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        //>> {
        //Implementar una llamada a una API de clima real.
        //self.temp = 25.5;

        // api real 
        let url = "https://api.open-meteo.com/v1/forecast?latitude=4.61&longitude=-74.08&current_weather=true";

        let resp = reqwest::get(url).await?.json::<ApiResponse>().await?;

        self.temp = resp.current_weather.temperature;

        // Opcional: cambiar el ícono según temperatura o condiciones
        self.icon = if self.temp > 30.0 {
            "🔥".to_string()
        } else if self.temp < 15.0 {
            "❄️".to_string()
        } else {
            "🌤️".to_string()
        };

        Ok(())
    }
    
}

/// Convierte un color de HSL a RGB.
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (((r_prime + m) * 255.0) as u8, ((g_prime + m) * 255.0) as u8, ((b_prime + m) * 255.0) as u8)
}

// --------------------------
// MAIN
// --------------------------
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(crossterm::terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;

    let mut sys = System::new_all();
    let mut weather = Weather::new();
    let mut metrics = SystemMetrics::new();
    let mut hue = 0.0;

    loop {
        let now = Local::now();
        sys.refresh_all();
        metrics.update(&sys);
        weather.update().await?;

        // Efecto de color
        hue = (hue + 0.3) % 360.0;
        let (r, g, b) = hsl_to_rgb(hue, 0.8, 0.6);
        let time_color = Color::Rgb { r, g, b };

        // Datos a mostrar
        let time_str = now.format("%H:%M:%S").to_string();
        let date_str = now.format("%a %d %b").to_string();
        let weather_str = format!("{} {:.1}°C", weather.icon, weather.temp);
        let (cols, _rows) = terminal::size()?;

        let metrics_str = format!(
            "CPU: {:>5.1}% | RAM: {:>5.1}% | DISK: {:>5.1}%",
            metrics.cpu_usage, metrics.mem_usage, metrics.disk_usage
        );
        // Rellenar con espacios para limpiar la línea y evitar artefactos visuales
        let padded_metrics_str = format!("{:<width$}", metrics_str, width = cols as usize);
        // Posicionamiento
        let weather_pos = cols - 10;
        let time_pos = weather_pos - 10;

        // Dibujado
        queue!(
            stdout,
            cursor::SavePosition,
            // Línea 1: Reloj + Clima
            cursor::MoveTo(time_pos, 0),
            SetForegroundColor(time_color),
            Print(time_str),
            cursor::MoveTo(weather_pos, 0),
            Print(weather_str),
            // Línea 2: Fecha
            cursor::MoveTo(time_pos, 1),
            Print(date_str),
            // Línea 3: Métricas del sistema (fijas)
            cursor::MoveTo(0, 2),
            Print(&padded_metrics_str),
            cursor::RestorePosition
        )?;

        stdout.flush()?;

        // Control de salida
        if crossterm::event::poll(Duration::from_millis(0))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.code == crossterm::event::KeyCode::Char('q') {
                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }

    stdout.execute(crossterm::terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    stdout.execute(cursor::Show)?;
    Ok(())
}

