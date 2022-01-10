#![allow(unused_imports)]
#![allow(unused_variables)]
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Condvar, Mutex};
use std::{cell::RefCell, env, sync::atomic::*, sync::Arc, thread, time::*};

use anyhow::bail;
use log::*;
use smol;
use url;

use embedded_svc::eth;
use embedded_svc::eth::Eth;
use embedded_svc::httpd::registry::*;
use embedded_svc::httpd::*;
use embedded_svc::io;
use embedded_svc::ipv4;
use embedded_svc::ping::Ping;
use embedded_svc::utils::anyerror::*;
use embedded_svc::wifi::*;

use esp_idf_svc::eth::*;
use esp_idf_svc::httpd as idf;
use esp_idf_svc::httpd::ServerRegistry;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sntp;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::wifi::*;

use esp_idf_sys;
use esp_idf_sys::esp;

use esp_idf_hal::delay;
use esp_idf_hal::gpio;
use esp_idf_hal::i2c;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;
use esp_idf_hal::ulp;

use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;

use ssd1306;
use ssd1306::mode::DisplayConfig;

const SSID: &str = "PJJ-KSR";
const PASS: &str = "pjj9794pjj9794!";

thread_local! {
    static TLS: RefCell<u32> = RefCell::new(13);
}

fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // let netif_stack = Arc::new(EspNetifStack::new()?);
    // let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    // let default_nvs = Arc::new(EspDefaultNvs::new()?);

    let spi = peripherals.spi1;
    heltec_hello_world(pins.gpio16, peripherals.i2c0, pins.gpio4, pins.gpio15)?;
    // let mut wifi = wifi(
    //     netif_stack.clone(),
    //     sys_loop_stack.clone(),
    //     default_nvs.clone(),
    // )
    // .unwrap();

    // test_tcp();
    // test_tcp_bind();
    // test_https_client();
    // drop(wifi);
    // info!("Wifi stopped");
    Ok(())
}

fn heltec_hello_world(
    rst: gpio::Gpio16<gpio::Unknown>,
    i2c: i2c::I2C0,
    sda: gpio::Gpio4<gpio::Unknown>,
    scl: gpio::Gpio15<gpio::Unknown>,
) -> Result<()> {
    info!("About to initialize the Heltec SSD1306 I2C LED driver");

    let config = <i2c::config::MasterConfig as Default>::default().baudrate(400.kHz().into());

    let di = ssd1306::I2CDisplayInterface::new(i2c::Master::<i2c::I2C0, _, _>::new(
        i2c,
        i2c::MasterPins { sda, scl },
        config,
    )?);

    let mut delay = delay::Ets;
    let mut reset = rst.into_output()?;

    reset.set_high()?;
    delay.delay_ms(1 as u32);

    reset.set_low()?;
    delay.delay_ms(10 as u32);

    reset.set_high()?;

    let mut display = ssd1306::Ssd1306::new(
        di,
        ssd1306::size::DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    AnyError::<display_interface::DisplayError>::wrap(|| {
        display.init()?;

        led_draw(&mut display)?;

        display.flush()
    })
}

fn led_draw<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget + Dimensions,
    D::Color: From<Rgb565>,
{
    display.clear(Rgb565::BLACK.into())?;

    Rectangle::new(display.bounding_box().top_left, display.bounding_box().size)
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLUE.into())
                .stroke_color(Rgb565::YELLOW.into())
                .stroke_width(1)
                .build(),
        )
        .draw(display)?;

    Text::new(
        "Hello Rust!",
        Point::new(10, (display.bounding_box().size.height - 10) as i32 / 2),
        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE.into()),
    )
    .draw(display)?;

    info!("LED rendering done");

    Ok(())
}
// fn test_https_client() -> Result<()> {
//     use embedded_svc::http::{self, client::*, status, Headers, Status};
//     use embedded_svc::io::Bytes;
//     use esp_idf_svc::http::client::*;

//     let url = String::from("https://google.com");
//     info!("About to fetch content from {}", url);

//     let mut client = EspHttpClient::new_default()?;

//     let response = client.get(&url)?.submit()?;

//     let body: Result<Vec<u8>, _> = Bytes::<_, 64>::new(response.reader()).take(3084).collect();
//     let body = body?;

//     info!(
//         "Body (truncated to 3K):\n{:?}",
//         String::from_utf8_lossy(&body).into_owned()
//     );

//     Ok(())
// }

// fn test_print() {
//     // Start simple
//     println!("Hello from Rust!");

//     // Check collections
//     let mut children = vec![];

//     children.push("foo");
//     children.push("bar");
//     println!("More complex print {:?}", children);
// }

// fn test_atomics() {
//     let a = AtomicUsize::new(0);
//     let v1 = a.compare_and_swap(0, 1, Ordering::SeqCst);
//     let v2 = a.swap(2, Ordering::SeqCst);

//     let (r1, r2) = unsafe {
//         // don't optimize our atomics out
//         let r1 = core::ptr::read_volatile(&v1);
//         let r2 = core::ptr::read_volatile(&v2);

//         (r1, r2)
//     };
//     println!("Result: {}, {}", r1, r2);
// }

// fn test_threads() {
//     let mut children = vec![];

//     println!("Rust main thread: {:?}", thread::current());
//     TLS.with(|tls| {
//         println!("Main TLS before change: {}", *tls.borrow());
//     });
//     TLS.with(|tls| *tls.borrow_mut() = 42);
//     TLS.with(|tls| {
//         println!("Main TLS after change: {}", *tls.borrow());
//     });

//     for i in 0..5 {
//         // Spin up another thread
//         children.push(thread::spawn(move || {
//             println!("This is thread number {}, {:?}", i, thread::current());
//             TLS.with(|tls| *tls.borrow_mut() = i);
//             TLS.with(|tls| {
//                 println!("Inner TLS: {}", *tls.borrow());
//             });
//         }));
//     }

//     println!(
//         "About to join the threads. If ESP-IDF was patched successfully, joining will NOT crash"
//     );

//     for child in children {
//         // Wait for the thread to finish. Returns a result.
//         let _ = child.join();
//     }

//     TLS.with(|tls| {
//         println!("Main TLS after threads: {}", *tls.borrow());
//     });

//     thread::sleep(Duration::from_secs(2));
//     println!("Joins were successful.");
// }

// fn test_tcp() -> Result<()> {
//     info!("About to open a TCP connection to 1.1.1.1 port 80");
//     let mut stream = TcpStream::connect("one.one.one.one:80")?;
//     let err = stream.try_clone();
//     if let Err(err) = err {
//         info!(
//             "Duplication of file descriptors does not work (yet) on the ESP-IDF, as expected: {}",
//             err
//         );
//     }
//     stream.write_all("GET / HTTP/1.0\n\n".as_bytes())?;
//     let mut result = Vec::new();
//     stream.read_to_end(&mut result)?;
//     info!(
//         "1.1.1.1 returned:\n=================\n{}\n=================\nSince it returned something, all is OK",
//         std::str::from_utf8(&result)?);
//     Ok(())
// }

// fn test_tcp_bind() -> Result<()> {
//     fn test_tcp_bind_accept() -> Result<()> {
//         info!("About to bind a simple echo service to port 8080");
//         let listener = TcpListener::bind("0.0.0.0:8080")?;
//         for stream in listener.incoming() {
//             match stream {
//                 Ok(stream) => {
//                     info!("Accepted client");
//                     thread::spawn(move || {
//                         test_tcp_bind_handle_client(stream);
//                     });
//                 }
//                 Err(e) => {
//                     error!("Error: {}", e);
//                 }
//             }
//         }
//         unreachable!()
//     }

//     fn test_tcp_bind_handle_client(mut stream: TcpStream) {
//         // read 20 bytes at a time from stream echoing back to stream
//         loop {
//             let mut read = [0; 128];
//             match stream.read(&mut read) {
//                 Ok(n) => {
//                     if n == 0 {
//                         // connection was closed
//                         break;
//                     }
//                     stream.write_all(&read[0..n]).unwrap();
//                 }
//                 Err(err) => {
//                     panic!("{}", err);
//                 }
//             }
//         }
//     }

//     thread::spawn(|| test_tcp_bind_accept().unwrap());
//     Ok(())
// }

// fn wifi(
//     netif_stack: Arc<EspNetifStack>,
//     sys_loop_stack: Arc<EspSysLoopStack>,
//     default_nvs: Arc<EspDefaultNvs>,
// ) -> Result<Box<EspWifi>> {
//     let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

//     info!("Wifi created, about to scan");

//     let ap_infos = wifi.scan()?;
//     let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);
//     let channel = if let Some(ours) = ours {
//         info!(
//             "Found configured access point {} on channel {}",
//             SSID, ours.channel
//         );
//         Some(ours.channel)
//     } else {
//         info!(
//             "Configured access point {} not found during scanning, will go with unknown channel",
//             SSID
//         );
//         None
//     };

//     wifi.set_configuration(&Configuration::Mixed(
//         ClientConfiguration {
//             ssid: SSID.into(),
//             password: PASS.into(),
//             channel,
//             ..Default::default()
//         },
//         AccessPointConfiguration {
//             ssid: "aptest".into(),
//             channel: channel.unwrap_or(1),
//             ..Default::default()
//         },
//     ))?;

//     info!("Wifi configuration set, about to get status");

//     let status = wifi.get_status();
//     if let Status(
//         ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
//         ApStatus::Started(ApIpStatus::Done),
//     ) = status
//     {
//         info!("Wifi connected");
//         ping(&ip_settings)?;
//     } else {
//         bail!("Unexpected Wifi status: {:?}", status);
//     }

//     Ok(wifi)
// }

// fn ping(ip_settings: &ipv4::ClientSettings) -> Result<()> {
//     info!("About to do some pings for {:?}", ip_settings);

//     let ping_summary =
//         ping::EspPing::default().ping(ip_settings.subnet.gateway, &Default::default())?;
//     if ping_summary.transmitted != ping_summary.received {
//         bail!(
//             "Pinging gateway {} resulted in timeouts",
//             ip_settings.subnet.gateway
//         );
//     }

//     info!("Pinging done");

//     Ok(())
// }
