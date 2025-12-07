#![windows_subsystem = "windows"]//指定子系统

use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::{Enigo, MouseButton, MouseControllable};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

static ENABLED: AtomicBool = AtomicBool::new(false);
static LEFT_MOUSE_DOWN: AtomicBool = AtomicBool::new(false);
static RIGHT_MOUSE_DOWN: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Serialize, Deserialize)]
struct KeyConfig {
    modifier_key: String,
    toggle_key: String,
    up_key: String,
    down_key: String,
    left_key: String,
    right_key: String,
    left_click_key: String,
    right_click_key: String,
    move_distance: u32,
}

#[derive(Debug)]
struct ParsedKeyConfig {
    modifier_key: Keycode,
    toggle_key: Keycode,
    up_key: Keycode,
    down_key: Keycode,
    left_key: Keycode,
    right_key: Keycode,
    left_click_key: Keycode,
    right_click_key: Keycode,
    move_distance: u32,
}

// 解析按键字符串,好像没啥意义，但是我就是写了
//   ┏┓　　┏┓
//  ┏┛┻━━━┛┻┓
//  ┃　　　　　┃ 　
//  ┃　　　━　　┃
//  ┃　┳┛　┗┳　┃
//  ┃　　　　　　┃
//  ┃　　　┻　　┃
//  ┃　　　　　 ┃
//  ┗━┓　　　┏━┛
//    ┃　　　┃ 　　　　　　　　
//    ┃　　　┃
//    ┃　　　┗━━━┓
//    ┃　　　　　 ┣┓
//    ┃　　　　　┏┛
//    ┗┓┓┏━┳┓┏┛
//     ┃┫┫ ┃┫┫
//     ┗┻┛ ┗┻┛
fn parse_keycode(key: &str) -> Option<Keycode> {
    match key.to_lowercase().as_str() {
        // 字母键
        "a" => Some(Keycode::A),
        "b" => Some(Keycode::B),
        "c" => Some(Keycode::C),
        "d" => Some(Keycode::D),
        "e" => Some(Keycode::E),
        "f" => Some(Keycode::F),
        "g" => Some(Keycode::G),
        "h" => Some(Keycode::H),
        "i" => Some(Keycode::I),
        "j" => Some(Keycode::J),
        "k" => Some(Keycode::K),
        "l" => Some(Keycode::L),
        "m" => Some(Keycode::M),
        "n" => Some(Keycode::N),
        "o" => Some(Keycode::O),
        "p" => Some(Keycode::P),
        "q" => Some(Keycode::Q),
        "r" => Some(Keycode::R),
        "s" => Some(Keycode::S),
        "t" => Some(Keycode::T),
        "u" => Some(Keycode::U),
        "v" => Some(Keycode::V),
        "w" => Some(Keycode::W),
        "x" => Some(Keycode::X),
        "y" => Some(Keycode::Y),
        "z" => Some(Keycode::Z),

        // 数字键
        "0" | "key0" => Some(Keycode::Key0),
        "1" | "key1" => Some(Keycode::Key1),
        "2" | "key2" => Some(Keycode::Key2),
        "3" | "key3" => Some(Keycode::Key3),
        "4" | "key4" => Some(Keycode::Key4),
        "5" | "key5" => Some(Keycode::Key5),
        "6" | "key6" => Some(Keycode::Key6),
        "7" | "key7" => Some(Keycode::Key7),
        "8" | "key8" => Some(Keycode::Key8),
        "9" | "key9" => Some(Keycode::Key9),

        // 功能键
        "f1" => Some(Keycode::F1),
        "f2" => Some(Keycode::F2),
        "f3" => Some(Keycode::F3),
        "f4" => Some(Keycode::F4),
        "f5" => Some(Keycode::F5),
        "f6" => Some(Keycode::F6),
        "f7" => Some(Keycode::F7),
        "f8" => Some(Keycode::F8),
        "f9" => Some(Keycode::F9),
        "f10" => Some(Keycode::F10),
        "f11" => Some(Keycode::F11),
        "f12" => Some(Keycode::F12),

        // 修饰键
        "lalt" => Some(Keycode::LAlt),
        "ralt" => Some(Keycode::RAlt),
        "lcontrol" | "lctrl" => Some(Keycode::LControl),
        "rcontrol" | "rctrl" => Some(Keycode::RControl),
        "lshift" => Some(Keycode::LShift),
        "rshift" => Some(Keycode::RShift),

        // 特殊键
        "space" => Some(Keycode::Space),
        "enter" | "return" => Some(Keycode::Enter),
        "tab" => Some(Keycode::Tab),
        "esc" | "escape" => Some(Keycode::Escape),
        "backspace" => Some(Keycode::Backspace),
        "capslock" => Some(Keycode::CapsLock),

        // 方向键
        "up" => Some(Keycode::Up),
        "down" => Some(Keycode::Down),
        "left" => Some(Keycode::Left),
        "right" => Some(Keycode::Right),

        // 符号键
        "`" | "grave" => Some(Keycode::Grave),
        "-" | "minus" => Some(Keycode::Minus),
        "=" | "equal" => Some(Keycode::Equal),
        "[" | "lbracket" => Some(Keycode::LeftBracket),
        "]" | "rbracket" => Some(Keycode::RightBracket),
        "\\" | "backslash" => Some(Keycode::BackSlash),
        ";" | "semicolon" => Some(Keycode::Semicolon),
        "'" | "apostrophe" => Some(Keycode::Apostrophe),
        "," | "comma" => Some(Keycode::Comma),
        "." | "period" => Some(Keycode::Dot),
        "/" | "slash" => Some(Keycode::Slash),

        _ => None,
    }
}

fn load_or_create_config() -> KeyConfig {
    // 从文件读取配置
    match fs::read_to_string("keyconfig.json") {
        Ok(content) => {
            match serde_json::from_str(&content) {
                Ok(config) => {
                    return config;
                }
                Err(_) => {
                    // 文件有问题，使用默认配置
                }
            }
        }
        Err(_) => {
            // 读取出问题，使用默认配置
        }
    }

    // 默认配置
    KeyConfig {
        modifier_key: "lalt".to_string(),
        toggle_key: "grave".to_string(),
        up_key: "i".to_string(),
        down_key: "k".to_string(),
        left_key: "j".to_string(),
        right_key: "l".to_string(),
        left_click_key: "lbracket".to_string(),
        right_click_key: "rbracket".to_string(),
        move_distance: 5,
    }
}

fn parse_config(config: &KeyConfig) -> Option<ParsedKeyConfig> {
    Some(ParsedKeyConfig {
        modifier_key: parse_keycode(&config.modifier_key)?,
        toggle_key: parse_keycode(&config.toggle_key)?,
        up_key: parse_keycode(&config.up_key)?,
        down_key: parse_keycode(&config.down_key)?,
        left_key: parse_keycode(&config.left_key)?,
        right_key: parse_keycode(&config.right_key)?,
        left_click_key: parse_keycode(&config.left_click_key)?,
        right_click_key: parse_keycode(&config.right_click_key)?,
        move_distance: config.move_distance,
    })
}

fn main() {
    let config = load_or_create_config();
    let parsed_config = match parse_config(&config) {
        Some(cfg) => cfg,
        None => {
            // 配置文件错误，使用默认配置
            return;
        }
    };

    let device_state = DeviceState::new();
    let mut enigo = Enigo::new();

    loop {
        let keys = device_state.get_keys();

        // 判断开关状态
        if keys.contains(&parsed_config.modifier_key) &&
            keys.contains(&parsed_config.toggle_key) {
            let current_state = ENABLED.load(Ordering::Relaxed);
            ENABLED.store(!current_state, Ordering::Relaxed);

            // 等待按键释放，避免重复触发
            while device_state.get_keys().contains(&parsed_config.toggle_key) {
                thread::sleep(Duration::from_millis(10));
            }

            // 重置鼠标状态
            if !ENABLED.load(Ordering::Relaxed) {
                // 释放鼠标按键
                if LEFT_MOUSE_DOWN.load(Ordering::Relaxed) {
                    enigo.mouse_up(MouseButton::Left);
                    LEFT_MOUSE_DOWN.store(false, Ordering::Relaxed);
                }
                if RIGHT_MOUSE_DOWN.load(Ordering::Relaxed) {
                    enigo.mouse_up(MouseButton::Right);
                    RIGHT_MOUSE_DOWN.store(false, Ordering::Relaxed);
                }
            }
        }

        // 开状态开启，处理鼠标控制
        if ENABLED.load(Ordering::Relaxed) {
            handle_mouse_control(&device_state, &mut enigo, &keys, &parsed_config);
        }

        // 简简蛋蛋给cpu减个负
        thread::sleep(Duration::from_millis(10));
    }
}

fn handle_mouse_control(
    _device_state: &DeviceState,
    enigo: &mut Enigo,
    keys: &[Keycode],
    config: &ParsedKeyConfig,
) {
    // 检查修饰键是否被按下
    if keys.contains(&config.modifier_key) {
        // 处理方向键移动鼠标
        let mut dx = 0;
        let mut dy = 0;
        let move_distance = config.move_distance; // 使用配置的移动距离

        // 计算移动距离
        if keys.contains(&config.left_key) {
            dx -= move_distance as i32;
        }
        if keys.contains(&config.right_key) {
            dx += move_distance as i32;
        }
        if keys.contains(&config.up_key) {
            dy -= move_distance as i32;
        }
        if keys.contains(&config.down_key) {
            dy += move_distance as i32;
        }

        // 执行移动
        if dx != 0 || dy != 0 {
            enigo.mouse_move_relative(dx, dy);
        }

        // 处理右键按住拖动
        if keys.contains(&config.right_click_key) {
            // 如果右键还没有按下，则按下右键
            if !RIGHT_MOUSE_DOWN.load(Ordering::Relaxed) {
                enigo.mouse_down(MouseButton::Right);
                RIGHT_MOUSE_DOWN.store(true, Ordering::Relaxed);
            }
        } else {
            // 如果右键已按下但不再按住，则释放右键
            if RIGHT_MOUSE_DOWN.load(Ordering::Relaxed) {
                enigo.mouse_up(MouseButton::Right);
                RIGHT_MOUSE_DOWN.store(false, Ordering::Relaxed);
            }
        }

        // 处理左键按住拖动
        if keys.contains(&config.left_click_key) {
            // 如果左键还没有按下，则按下左键
            if !LEFT_MOUSE_DOWN.load(Ordering::Relaxed) {
                enigo.mouse_down(MouseButton::Left);
                LEFT_MOUSE_DOWN.store(true, Ordering::Relaxed);
            }
        } else {
            // 如果左键已按下但不再按住，则释放左键
            if LEFT_MOUSE_DOWN.load(Ordering::Relaxed) {
                enigo.mouse_up(MouseButton::Left);
                LEFT_MOUSE_DOWN.store(false, Ordering::Relaxed);
            }
        }

        // 短暂延迟防止频繁操作
        thread::sleep(Duration::from_millis(10));
    } else {
        // 如果修饰键未按下，释放所有可能按下的鼠标按键
        if LEFT_MOUSE_DOWN.load(Ordering::Relaxed) {
            enigo.mouse_up(MouseButton::Left);
            LEFT_MOUSE_DOWN.store(false, Ordering::Relaxed);
        }
        if RIGHT_MOUSE_DOWN.load(Ordering::Relaxed) {
            enigo.mouse_up(MouseButton::Right);
            RIGHT_MOUSE_DOWN.store(false, Ordering::Relaxed);
        }
    }
}