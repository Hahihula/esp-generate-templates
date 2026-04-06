//INCLUDEFILE option("embassy")
//INCLUDE_AS src/bin/main.rs
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;

//IF option("ble-trouble") || option("ble-bleps")
use esp_radio::ble::controller::BleConnector;
//ENDIF
//IF option("ble-trouble")
use bt_hci::controller::ExternalController;
use trouble_host::prelude::*;
//ENDIF

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

//IF option("defmt")
//IF !option("probe-rs")
//+use esp_println as _;
//ENDIF
//+use defmt::info;
//IF !option("panic-handler")
//+use defmt::error;
//ENDIF !option("panic-handler")
//ELIF option("log")
use log::info;
//IF !option("panic-handler")
use log::error;
//ENDIF !option("panic-handler")
//ELIF option("probe-rs") // without defmt
//+use rtt_target::rprintln;
//ENDIF !defmt

//IF !option("panic-handler")
//IF option("defmt") || option("log")
//+#[panic_handler]
//+fn panic(panic_info: &core::panic::PanicInfo) -> ! {
//+    error!("{}", panic_info);
//+    loop {}
//+}
//ELIF option("probe-rs")
//+#[panic_handler]
//+fn panic(panic_info: &core::panic::PanicInfo) -> ! {
//+    rprintln!("{}", panic_info);
//+    loop {}
//+}
//ELSE
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
//ENDIF
//ELIF option("esp-backtrace")
//+use esp_backtrace as _;
//ELIF option("panic-rtt-target")
//+use panic_rtt_target as _;
//ENDIF

//IF option("alloc")
extern crate alloc;
//ENDIF

//IF option("ble-trouble")
const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;
//ENDIF

//IF option("display") || option("demo")
//+use core::fmt::Write;
//+use embedded_graphics::{
//+    pixelcolor::Rgb565,
//+    prelude::*,
//+};
//+use embedded_hal::delay::DelayNs;
//+use esp_hal::{
//+    gpio::Level,
//+    gpio::Output,
//+    gpio::OutputConfig,
//+    spi::master::Spi,
//+    time::Rate,
//+};
//+#[path = "../ferris_bitmap.rs"]
//+mod ferris_bitmap;
//+use crate::ferris_bitmap::{QR_CODE, QR_WIDTH, QR_HEIGHT, QR_BYTES_PER_ROW, FERRIS_DATA, FERRIS_WIDTH, FERRIS_HEIGHT};
//ENDIF

//IF option("touch") || option("imu") || option("demo")
//+use embedded_hal_bus::i2c::RefCellDevice;
//+use core::cell::RefCell;
//+use embedded_hal::i2c::I2c;
//+use esp_hal::i2c::master::I2c as I2cMaster;
//ENDIF

//IF option("touch")
//+use axs5106l::{Axs5106l, Rotation};
//ENDIF

//IF option("temp-sensor") || option("demo")
//+use esp_hal::tsens::{TemperatureSensor, Config};
//ENDIF

//IF option("display") || option("demo")
//+const LCD_CMD_SLPOUT: u8 = 0x11;
//+const LCD_CMD_DISPON: u8 = 0x29;
//+const LCD_CMD_MADCTL: u8 = 0x36;
//+const LCD_CMD_COLMOD: u8 = 0x3A;
//+const LCD_CMD_CASET: u8 = 0x2A;
//+const LCD_CMD_RASET: u8 = 0x2B;
//+const LCD_CMD_RAMWR: u8 = 0x2C;
//+
//+const LCD_H_RES: usize = 172;
//+const LCD_V_RES: usize = 320;
//+const LCD_X_OFFSET: usize = 34;
//+const LCD_BUFFER_SIZE: usize = LCD_H_RES * LCD_V_RES;
//+
//+const DISPLAY_WIDTH: u16 = LCD_H_RES as u16;
//+const DISPLAY_HEIGHT: u16 = LCD_V_RES as u16;
//+
//+const QR_SCALE: usize = 4;
//+
//+fn lcd_write_cmd(spi: &mut Spi<'_, esp_hal::Blocking>, cs: &mut Output, dc: &mut Output, cmd: u8) {
//+    cs.set_low();
//+    dc.set_low();
//+    spi.write(&[cmd]).ok();
//+    cs.set_high();
//+}
//+
//+fn lcd_write_data(spi: &mut Spi<'_, esp_hal::Blocking>, cs: &mut Output, dc: &mut Output, data: &[u8]) {
//+    cs.set_low();
//+    dc.set_high();
//+    spi.write(data).ok();
//+    cs.set_high();
//+}
//+
//+fn lcd_init(spi: &mut Spi<'_, esp_hal::Blocking>, cs: &mut Output, dc: &mut Output, rst: &mut Output, delay: &mut impl DelayNs) {
//+    rst.set_low();
//+    delay.delay_ms(10);
//+    rst.set_high();
//+    delay.delay_ms(10);
//+
//+    lcd_write_cmd(spi, cs, dc, LCD_CMD_SLPOUT);
//+    delay.delay_ms(120);
//+
//+    lcd_write_cmd(spi, cs, dc, LCD_CMD_MADCTL);
//+    lcd_write_data(spi, cs, dc, &[0x00]);
//+
//+    lcd_write_cmd(spi, cs, dc, LCD_CMD_COLMOD);
//+    lcd_write_data(spi, cs, dc, &[0x05]);
//+
//+    lcd_write_cmd(spi, cs, dc, 0xDF);
//+    lcd_write_data(spi, cs, dc, &[0x98, 0x53]);
//+    lcd_write_cmd(spi, cs, dc, 0xB2);
//+    lcd_write_data(spi, cs, dc, &[0x23]);
//+    lcd_write_cmd(spi, cs, dc, 0xB7);
//+    lcd_write_data(spi, cs, dc, &[0x00, 0x47, 0x00, 0x6F]);
//+    lcd_write_cmd(spi, cs, dc, 0xBB);
//+    lcd_write_data(spi, cs, dc, &[0x1C, 0x1A, 0x55, 0x73, 0x63, 0xF0]);
//+    lcd_write_cmd(spi, cs, dc, 0xC0);
//+    lcd_write_data(spi, cs, dc, &[0x44, 0xA4]);
//+    lcd_write_cmd(spi, cs, dc, 0xC1);
//+    lcd_write_data(spi, cs, dc, &[0x16]);
//+    lcd_write_cmd(spi, cs, dc, 0xC3);
//+    lcd_write_data(spi, cs, dc, &[0x7D, 0x07, 0x14, 0x06, 0xCF, 0x71, 0x72, 0x77]);
//+    lcd_write_cmd(spi, cs, dc, 0xC8);
//+    lcd_write_data(spi, cs, dc, &[
//+        0x3F, 0x32, 0x29, 0x29, 0x27, 0x2B, 0x27, 0x28, 0x28, 0x26, 0x25, 0x17, 0x12, 0x0D, 0x04, 0x00,
//+        0x3F, 0x32, 0x29, 0x29, 0x27, 0x2B, 0x27, 0x28, 0x28, 0x26, 0x25, 0x17, 0x12, 0x0D, 0x04, 0x00
//+    ]);
//+    lcd_write_cmd(spi, cs, dc, 0xD0);
//+    lcd_write_data(spi, cs, dc, &[0x04, 0x06, 0x6B, 0x0F, 0x00]);
//+    lcd_write_cmd(spi, cs, dc, LCD_CMD_DISPON);
//+    delay.delay_ms(20);
//+
//+    lcd_write_cmd(spi, cs, dc, 0x21);
//+    delay.delay_ms(10);
//+}
//+
//+fn lcd_draw_framebuffer(spi: &mut Spi<'_, esp_hal::Blocking>, cs: &mut Output, dc: &mut Output, fb: &[Rgb565]) {
//+    lcd_write_cmd(spi, cs, dc, LCD_CMD_CASET);
//+    lcd_write_data(spi, cs, dc, &[
//+        0x00, LCD_X_OFFSET as u8,
//+        ((LCD_H_RES + LCD_X_OFFSET - 1) >> 8) as u8,
//+        (LCD_H_RES + LCD_X_OFFSET - 1) as u8,
//+    ]);
//+
//+    lcd_write_cmd(spi, cs, dc, LCD_CMD_RASET);
//+    lcd_write_data(spi, cs, dc, &[
//+        0x00, 0x00,
//+        (LCD_V_RES >> 8) as u8,
//+        (LCD_V_RES - 1) as u8,
//+    ]);
//+
//+    lcd_write_cmd(spi, cs, dc, LCD_CMD_RAMWR);
//+
//+    cs.set_low();
//+    dc.set_high();
//+
//+    let mut pixel_bytes = [0u8; 512];
//+    let mut i = 0;
//+
//+    for pixel in fb.iter() {
//+        let r = pixel.r() as u16;
//+        let g = pixel.g() as u16;
//+        let b = pixel.b() as u16;
//+        let rgb = (r << 11) | (g << 5) | b;
//+        pixel_bytes[i] = (rgb >> 8) as u8;
//+        pixel_bytes[i + 1] = rgb as u8;
//+        i += 2;
//+
//+        if i >= 512 {
//+            spi.write(&pixel_bytes[..512]).ok();
//+            i = 0;
//+        }
//+    }
//+
//+    if i > 0 {
//+        spi.write(&pixel_bytes[..i]).ok();
//+    }
//+
//+    cs.set_high();
//+}
//+
//+struct FrameBuffer {
//+    pixels: [Rgb565; LCD_BUFFER_SIZE],
//+}
//+
//+impl FrameBuffer {
//+    fn new() -> Self {
//+        Self { pixels: [Rgb565::WHITE; LCD_BUFFER_SIZE] }
//+    }
//+
//+    fn clear(&mut self, color: Rgb565) {
//+        self.pixels.fill(color);
//+    }
//+
//+    fn draw_circle(&mut self, cx: i32, cy: i32, r: u32, color: Rgb565) {
//+        let r = r as i32;
//+        for y in -r..=r {
//+            for x in -r..=r {
//+                if x*x + y*y <= r*r {
//+                    let px = cx + x;
//+                    let py = cy + y;
//+                    if px >= 0 && py >= 0 && (px as usize) < LCD_H_RES && (py as usize) < LCD_V_RES {
//+                        self.pixels[(py as usize) * LCD_H_RES + (px as usize)] = color;
//+                    }
//+                }
//+            }
//+        }
//+    }
//+
//+    fn as_slice(&self) -> &[Rgb565] {
//+        &self.pixels
//+    }
//+
//+    fn pixel_mut(&mut self, x: i32, y: i32) -> Option<&mut Rgb565> {
//+        if x >= 0 && y >= 0 && (x as usize) < LCD_H_RES && (y as usize) < LCD_V_RES {
//+            Some(&mut self.pixels[(y as usize) * LCD_H_RES + (x as usize)])
//+        } else {
//+            None
//+        }
//+    }
//+}
//+
//+impl embedded_graphics::prelude::DrawTarget for FrameBuffer {
//+    type Color = Rgb565;
//+    type Error = core::convert::Infallible;
//+
//+    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
//+    where
//+        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
//+    {
//+        for embedded_graphics::Pixel(point, color) in pixels {
//+            if let Some(p) = self.pixel_mut(point.x, point.y) {
//+                *p = color;
//+            }
//+        }
//+        Ok(())
//+    }
//+
//+    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
//+        self.clear(color);
//+        Ok(())
//+    }
//+}
//+
//+impl embedded_graphics::prelude::OriginDimensions for FrameBuffer {
//+    fn size(&self) -> embedded_graphics::prelude::Size {
//+        embedded_graphics::prelude::Size::new(LCD_H_RES as u32, LCD_V_RES as u32)
//+    }
//+}
//+
//+fn draw_ferris(fb: &mut FrameBuffer, start_x: i32, start_y: i32) {
//+    for y in 0..FERRIS_HEIGHT {
//+        for x in 0..FERRIS_WIDTH {
//+            let rgb565 = FERRIS_DATA[y * FERRIS_WIDTH + x];
//+            if rgb565 == 0xFFFF {
//+                continue;
//+            }
//+            let color = Rgb565::new(
//+                ((rgb565 >> 11) & 0x1F) as u8,
//+                ((rgb565 >> 5) & 0x3F) as u8,
//+                (rgb565 & 0x1F) as u8,
//+            );
//+            let px = start_x + (x as i32) * 2;
//+            let py = start_y + (y as i32) * 2;
//+            if let Some(p) = fb.pixel_mut(px, py) { *p = color; }
//+            if let Some(p) = fb.pixel_mut(px + 1, py) { *p = color; }
//+            if let Some(p) = fb.pixel_mut(px, py + 1) { *p = color; }
//+            if let Some(p) = fb.pixel_mut(px + 1, py + 1) { *p = color; }
//+        }
//+    }
//+}
//+
//+fn draw_qr_code(fb: &mut FrameBuffer, start_x: i32, start_y: i32) {
//+    for y in 0..QR_HEIGHT {
//+        for x in 0..QR_WIDTH {
//+            let byte_idx = y * QR_BYTES_PER_ROW + (x / 8);
//+            let bit_idx = 7 - (x % 8);
//+            if byte_idx < QR_CODE.len() {
//+                let is_white = (QR_CODE[byte_idx] & (1 << bit_idx)) != 0;
//+                let color = if is_white { Rgb565::WHITE } else { Rgb565::BLACK };
//+                for dy in 0..QR_SCALE {
//+                    for dx in 0..QR_SCALE {
//+                        if let Some(p) = fb.pixel_mut(
//+                            start_x + (x * QR_SCALE + dx) as i32,
//+                            start_y + (y * QR_SCALE + dy) as i32,
//+                        ) {
//+                            *p = color;
//+                        }
//+                    }
//+                }
//+            }
//+        }
//+    }
//+}
//ENDIF

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(clippy::large_stack_frames)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    //REPLACE generate-version generate-version
    // generator version: generate-version
    //REPLACE generate-parameters generate-parameters
    // generator parameters: generate-parameters

    //IF option("probe-rs")
    //IF option("defmt")
    rtt_target::rtt_init_defmt!();
    //ELSE
    rtt_target::rtt_init_print!();
    //ENDIF
    //ELIF option("log")
    esp_println::logger::init_logger_from_env();
    //ENDIF

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    //IF option("module-selected")
    // Reserved GPIOs - directly connected to flash/PSRAM
    //REPLACE __RESERVED_GPIO_CODE__ reserved_gpio_code
    __RESERVED_GPIO_CODE__
    //ENDIF

    //IF option("alloc")
    //REPLACE 65536 max-dram2-uninit
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);
    //IF option("wifi") && (option("ble-bleps") || option("ble-trouble"))
    // COEX needs more RAM - so we've added some more
    esp_alloc::heap_allocator!(size: 64 * 1024);
    //ENDIF
    //ENDIF alloc

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    //IF option("esp32") || option("esp32s2") || option("esp32s3")
    esp_rtos::start(timg0.timer0);
    //ELSE
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    //ENDIF

    //IF option("display")
    let mut spi = Spi::new(
        peripherals.SPI2,
        esp_hal::spi::master::Config::default()
            .with_frequency(Rate::from_mhz(80))
            .with_mode(esp_hal::spi::Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO1)
    .with_mosi(peripherals.GPIO2);

    let mut cs = Output::new(peripherals.GPIO14, Level::High, OutputConfig::default());
    let mut dc = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());
    let mut rst = Output::new(peripherals.GPIO22, Level::High, OutputConfig::default());
    let mut backlight = Output::new(peripherals.GPIO23, Level::Low, OutputConfig::default());
    backlight.set_high();

    lcd_init(&mut spi, &mut cs, &mut dc, &mut rst, &mut esp_hal::delay::Delay::new());
    //ENDIF

    //IF option("touch") || option("imu") || option("demo")
    let i2c0 = I2cMaster::new(
        peripherals.I2C0,
        esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO18)
    .with_scl(peripherals.GPIO19);

    let i2c0_refcell = RefCell::new(i2c0);
    //ENDIF

    //IF option("touch") || option("demo")
    let touch_i2c = RefCellDevice::new(&i2c0_refcell);
    let touch_rst = Output::new(peripherals.GPIO20, Level::High, OutputConfig::default());
    let mut touch = Axs5106l::new(touch_i2c, touch_rst, DISPLAY_WIDTH, DISPLAY_HEIGHT, Rotation::Rotate0);
    match touch.init(&mut esp_hal::delay::Delay::new()) {
        Ok(_) => {
            //IF option("defmt") || option("log")
            info!("Touch initialized successfully!");
            //ELIF option("probe-rs")
            //+rprintln!("Touch initialized successfully!");
            //ENDIF
        }
        Err(e) => {
            //IF option("defmt") || option("log")
            info!("Touch init error: {:?}", e);
            //ELIF option("probe-rs")
            //+rprintln!("Touch init error: {:?}", e);
            //ENDIF
        }
    }
    //ENDIF

    //IF option("imu") || option("demo")
    let mut imu_i2c = RefCellDevice::new(&i2c0_refcell);
    const IMU_ADDR: u8 = 0x6B;
    imu_i2c.write(IMU_ADDR, &[0x60, 0xB0]).ok();
    esp_hal::delay::Delay::new().delay_ms(30);
    imu_i2c.write(IMU_ADDR, &[0x02, 0x60]).ok();
    imu_i2c.write(IMU_ADDR, &[0x08, 0x03]).ok();
    imu_i2c.write(IMU_ADDR, &[0x03, 0x15]).ok();
    imu_i2c.write(IMU_ADDR, &[0x04, 0x55]).ok();
    //IF option("defmt") || option("log")
    info!("IMU initialized at 0x{:02X}!", IMU_ADDR);
    //ELIF option("probe-rs")
    //+rprintln!("IMU initialized at 0x{:02X}!", IMU_ADDR);
    //ENDIF
    //ENDIF

    //IF option("temp-sensor") || option("demo")
    let temp_sensor = TemperatureSensor::new(peripherals.TSENS, Config::default()).unwrap();
    //IF option("defmt") || option("log")
    info!("Temperature sensor initialized");
    //ELIF option("probe-rs")
    //+rprintln!("Temperature sensor initialized");
    //ENDIF
    //ENDIF

    //IF option("display")
    let mut fb = FrameBuffer::new();
    //ENDIF

    //IF option("demo")
    let button = esp_hal::gpio::Input::new(
        peripherals.GPIO9,
        esp_hal::gpio::InputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );

    let mut current_page: u8 = 0;
    let mut button_pressed = false;

    let mut imu_ax: i16 = 0;
    let mut imu_ay: i16 = 0;
    let mut imu_az: i16 = 0;
    let mut imu_gx: i16 = 0;
    let mut imu_gy: i16 = 0;
    let mut imu_gz: i16 = 0;
    let mut temperature: f32 = 0.0;

    use embedded_graphics::mono_font::MonoTextStyle;
    use embedded_graphics::text::Text;
    let text_style = MonoTextStyle::new(&embedded_graphics::mono_font::ascii::FONT_6X9, Rgb565::BLACK);
    let title_style = MonoTextStyle::new(&embedded_graphics::mono_font::ascii::FONT_6X9, Rgb565::BLUE);

    fb.clear(Rgb565::WHITE);
    Text::new("Touch Demo", Point::new(45, 50), title_style).draw(&mut fb).ok();
    Text::new("Touch the screen!", Point::new(30, 100), text_style).draw(&mut fb).ok();
    Text::new("Push button for", Point::new(25, 280), text_style).draw(&mut fb).ok();
    Text::new("IMU demo ->", Point::new(35, 295), text_style).draw(&mut fb).ok();
    lcd_draw_framebuffer(&mut spi, &mut cs, &mut dc, fb.as_slice());
    //ENDIF

    //IF option("defmt") || option("log")
    info!("Embassy initialized!");
    //ELIF option("probe-rs")
    //+rprintln!("Embassy initialized!");
    //ENDIF

    //IF option("ble-trouble") || option("ble-bleps") || option("wifi")
    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    //ENDIF
    //IF option("wifi")
    let (mut _wifi_controller, _interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");
    //ENDIF
    //IF option("ble-trouble")
    let transport = BleConnector::new(&radio_init, peripherals.BT, Default::default()).unwrap();
    let ble_controller = ExternalController::<_, 1>::new(transport);
    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let _stack = trouble_host::new(ble_controller, &mut resources);
    //ELIF option("ble-bleps")
    let _connector = BleConnector::new(&radio_init, peripherals.BT, Default::default());
    //ENDIF

    let _ = spawner;

    //IF option("demo")
    loop {
        if button.is_low() && !button_pressed {
            button_pressed = true;
            current_page = (current_page + 1) % 4;

            match current_page {
                0 => {
                    fb.clear(Rgb565::WHITE);
                    Text::new("Touch Demo", Point::new(45, 50), title_style).draw(&mut fb).ok();
                    Text::new("Touch the screen!", Point::new(30, 100), text_style).draw(&mut fb).ok();
                    Text::new("Push button for", Point::new(25, 280), text_style).draw(&mut fb).ok();
                    Text::new("IMU demo ->", Point::new(35, 295), text_style).draw(&mut fb).ok();
                    //IF option("defmt") || option("log")
                    info!("Switched to touch page");
                    //ELIF option("probe-rs")
                    //+rprintln!("Switched to touch page");
                    //ENDIF
                }
                1 => {
                    //IF option("defmt") || option("log")
                    info!("Switched to IMU page");
                    //ELIF option("probe-rs")
                    //+rprintln!("Switched to IMU page");
                    //ENDIF
                }
                2 => {
                    temperature = temp_sensor.get_temperature().to_celsius();
                    fb.clear(Rgb565::WHITE);
                    Text::new("System Info", Point::new(35, 30), title_style).draw(&mut fb).ok();
                    Text::new("Chip: ESP32-C6", Point::new(10, 70), text_style).draw(&mut fb).ok();
                    Text::new("CPU: 160 MHz", Point::new(10, 90), text_style).draw(&mut fb).ok();
                    Text::new("Flash: 8 MB", Point::new(10, 110), text_style).draw(&mut fb).ok();
                    Text::new("RAM: 512 KB", Point::new(10, 130), text_style).draw(&mut fb).ok();

                    let mut temp_buf: heapless::String<16> = heapless::String::new();
                    core::write!(&mut temp_buf, "Temp: {:.1} C", temperature).ok();
                    Text::new(&temp_buf, Point::new(10, 150), text_style).draw(&mut fb).ok();

                    Text::new("Display:", Point::new(10, 190), text_style).draw(&mut fb).ok();
                    Text::new("  172x320 RGB565", Point::new(10, 210), text_style).draw(&mut fb).ok();
                    Text::new("Sensors:", Point::new(10, 250), text_style).draw(&mut fb).ok();
                    Text::new("  Touch: AXS5106L", Point::new(10, 270), text_style).draw(&mut fb).ok();
                    Text::new("  IMU: QMI8658", Point::new(10, 290), text_style).draw(&mut fb).ok();
                    //IF option("defmt") || option("log")
                    info!("Switched to system page");
                    //ELIF option("probe-rs")
                    //+rprintln!("Switched to system page");
                    //ENDIF
                }
                3 => {
                    fb.clear(Rgb565::WHITE);
                    Text::new("About", Point::new(60, 15), title_style).draw(&mut fb).ok();
                    draw_ferris(&mut fb, 6, -10);
                    draw_qr_code(&mut fb, 16, 115);
                    Text::new("Scan QR for", Point::new(50, 270), text_style).draw(&mut fb).ok();
                    Text::new("GitHub repo!", Point::new(50, 285), text_style).draw(&mut fb).ok();
                    Text::new("Push button", Point::new(30, 305), text_style).draw(&mut fb).ok();
                    Text::new("for touch demo", Point::new(25, 315), text_style).draw(&mut fb).ok();
                    //IF option("defmt") || option("log")
                    info!("Switched to about page");
                    //ELIF option("probe-rs")
                    //+rprintln!("Switched to about page");
                    //ENDIF
                }
                _ => {}
            }
            lcd_draw_framebuffer(&mut spi, &mut cs, &mut dc, fb.as_slice());
        }
        if button.is_high() {
            button_pressed = false;
        }

        match current_page {
            0 => {
                match touch.get_touch_data() {
                    Ok(Some(touch_data)) => {
                        if let Some(coord) = touch_data.first_touch() {
                            let screen_x = coord.x as i32;
                            let screen_y = coord.y as i32;
                            fb.draw_circle(screen_x, screen_y, 5, Rgb565::RED);
                            lcd_draw_framebuffer(&mut spi, &mut cs, &mut dc, fb.as_slice());
                        }
                    }
                    Ok(None) => {}
                    Err(_) => {}
                }
            }
            1 => {
                let mut status = [0u8; 1];
                if imu_i2c.write_read(IMU_ADDR, &[0x2E], &mut status).is_ok() {
                    if status[0] & 0x03 != 0 {
                        let mut imu_buf = [0u8; 12];
                        if imu_i2c.write_read(IMU_ADDR, &[0x35], &mut imu_buf).is_ok() {
                            imu_ax = i16::from_le_bytes([imu_buf[0], imu_buf[1]]);
                            imu_ay = i16::from_le_bytes([imu_buf[2], imu_buf[3]]);
                            imu_az = i16::from_le_bytes([imu_buf[4], imu_buf[5]]);
                            imu_gx = i16::from_le_bytes([imu_buf[6], imu_buf[7]]);
                            imu_gy = i16::from_le_bytes([imu_buf[8], imu_buf[9]]);
                            imu_gz = i16::from_le_bytes([imu_buf[10], imu_buf[11]]);

                            fb.clear(Rgb565::WHITE);
                            Text::new("IMU Demo", Point::new(50, 30), title_style).draw(&mut fb).ok();
                            Text::new("Accel:", Point::new(10, 70), text_style).draw(&mut fb).ok();
                            let mut buf: heapless::String<16> = heapless::String::new();
                            core::write!(&mut buf, "  X: {:6}", imu_ax).ok();
                            Text::new(&buf, Point::new(10, 90), text_style).draw(&mut fb).ok();
                            let mut buf: heapless::String<16> = heapless::String::new();
                            core::write!(&mut buf, "  Y: {:6}", imu_ay).ok();
                            Text::new(&buf, Point::new(10, 110), text_style).draw(&mut fb).ok();
                            let mut buf: heapless::String<16> = heapless::String::new();
                            core::write!(&mut buf, "  Z: {:6}", imu_az).ok();
                            Text::new(&buf, Point::new(10, 130), text_style).draw(&mut fb).ok();
                            Text::new("Gyro:", Point::new(10, 170), text_style).draw(&mut fb).ok();
                            let mut buf: heapless::String<16> = heapless::String::new();
                            core::write!(&mut buf, "  X: {:6}", imu_gx).ok();
                            Text::new(&buf, Point::new(10, 190), text_style).draw(&mut fb).ok();
                            let mut buf: heapless::String<16> = heapless::String::new();
                            core::write!(&mut buf, "  Y: {:6}", imu_gy).ok();
                            Text::new(&buf, Point::new(10, 210), text_style).draw(&mut fb).ok();
                            let mut buf: heapless::String<16> = heapless::String::new();
                            core::write!(&mut buf, "  Z: {:6}", imu_gz).ok();
                            Text::new(&buf, Point::new(10, 230), text_style).draw(&mut fb).ok();
                            Text::new("Push button", Point::new(30, 280), text_style).draw(&mut fb).ok();
                            Text::new("for touch demo", Point::new(25, 295), text_style).draw(&mut fb).ok();
                            lcd_draw_framebuffer(&mut spi, &mut cs, &mut dc, fb.as_slice());
                        }
                    }
                }
            }
            2 => {}
            3 => {}
            _ => {}
        }

        Timer::after(Duration::from_millis(50)).await;
    }
    //ELSE
    loop {
        //IF option("defmt") || option("log")
        info!("Hello world!");
        //ELIF option("probe-rs")
        rprintln!("Hello world!");
        //ENDIF
        Timer::after(Duration::from_secs(1)).await;
    }
    //ENDIF demo

    //REPLACE {current-version} esp-hal-version-full
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v{current-version}/examples
}