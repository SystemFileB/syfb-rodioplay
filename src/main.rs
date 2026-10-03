use anyhow::{Context, Result};
use clap::{ArgAction, Command, arg, builder::styling, value_parser};
use crossterm::{cursor::MoveUp, execute, style::Print};
use rodio::{Decoder, DeviceSinkBuilder, Player, Source};
use std::cmp::max;
use std::env::current_exe;
use std::fs::File;
use std::io::{BufReader, BufWriter, stderr, stdout};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::sleep;
use std::time::{Duration, Instant};

// TODO: 1. 将源代码上传 GitHub 并写 README
// TODO: 2. 为 advmix 子命令写播放 + 输出逻辑

const CLI_STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Blue.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

fn cli() -> Result<Command> {
    let bin_name: String = current_exe()
        .context("无法获取可执行文件名")?
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("rodioplay")
        .to_string();
    let version_str: String = format!(
        "1.{}{:02}{}{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR")
            .parse::<i32>()
            .context(format!(
                "无法将小版本号转为 32 位整数, 当前版本号为: {}",
                env!("CARGO_PKG_VERSION")
            ))?,
        match env!("CARGO_PKG_VERSION_PATCH") {
            "0" => String::new(),
            x => format!(".{}", x),
        },
        match env!("CARGO_PKG_VERSION_PRE") {
            "" => String::new(),
            x => format!(".{}", x),
        }
    );

    Ok(Command::new("SyFB's rodioplay")
        .styles(CLI_STYLES)
        .bin_name(bin_name)
        .version(format!(
            "{}\n\
            使用 Mozilla 公共许可证 2.0 (Mozilla Public License 2.0, MPL 2.0) 授权\n\
            \n\
            By SystemFileB",
            version_str
        ))
        .before_help(format!("SyFB's rodioplay {}\nBy SystemFileB", version_str))
        .about("一个特别简单的音频播放器, 用于给作者研究 rodio 库")
        .arg(
            arg!(-s --speed <SPEED>   "播放时的速度 (例如: 1.0 为原速, 2.0 为 2 倍速)")
                .value_parser(value_parser!(f32))
                .default_value("1"),
        )
        .arg(
            arg!(-v --volume <VOLUME> "播放时的音量 (例如: 1.0 为 100 % 音量, 2.0 为 200 % 音量)")
                .value_parser(value_parser!(f64))
                .default_value("1"),
        )
        .arg(arg!(-q --quiet           "安静模式, 播放时不输出任何信息").default_value("false"))
        .arg(arg!(<FILE> "需要播放的音频文件路径").required(true))
        .arg_required_else_help(true)
        .subcommand(
            Command::new("moo")
                .hide(true)
                .before_help(format!("SyFB's rodioplay {}\nBy SystemFileB", version_str))
                .about("彩蛋")
                .arg(arg!(-v --verbose "查看更多").action(ArgAction::Count))
                .arg(arg!(-q --quiet   "查看更少").action(ArgAction::Count))
                .after_help("-v 和 -q 参数可以叠加使用, 一个 -v 和一个 -q 会抵消掉"),
        )
        .subcommand(
            Command::new("mix")
                .before_help(format!("SyFB's rodioplay {}\nBy SystemFileB", version_str))
                .about("同时播放多个音频, 不关心音频的播放情况")
                .arg(
                    arg!(-t --time <TIME> "播放每个音频的间隔 (单位: 秒)")
                        .value_parser(value_parser!(f64))
                        .default_value("0"),
                )
                .arg(
                    arg!(<FILE> "需要播放的音频文件路径, 可以有多个")
                        .action(ArgAction::Append)
                        .required(true),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("advmix")
                .before_help(format!("SyFB's rodioplay {}\n\
                                      By SystemFileB", version_str))
                .about("同时播放多个音频，可以显示每个音频的播放进度")
                .arg(
                    arg!(-s --speed <SPEED>   "播放时的速度 (例如: 1.0 为原速, 2.0 为 2 倍速)")
                        .value_parser(value_parser!(f32))
                        .default_value("1"),
                )
                .arg(
                    arg!(-v --volume <VOLUME> "播放时的音量 (例如: 1.0 为 100 % 音量, 2.0 为 200 % 音量)")
                        .value_parser(value_parser!(f64))
                        .default_value("1"),
                )
                .arg(
                    arg!(-t --time <TIME> "播放每个音频的间隔 (单位: 秒)\n\
                                           警告: 音频播放时机检查会每 10ms 进行一次, 所以我不保证这里的间隔时间\n      是 100% 准确的")
                        .value_parser(value_parser!(f64))
                        .default_value("0"),
                )
                .arg(
                    arg!(<FILE> "需要播放的音频文件路径, 可以有多个")
                        .action(ArgAction::Append)
                        .required(true),
                )
                .arg_required_else_help(true)
        )
        .subcommand_negates_reqs(true)
        .after_help("本 rodioplay 没有超级牛力"))
}

fn format_seconds(secs: f64) -> String {
    if secs == -1.0 {
        return String::from("?");
    }

    let total_centis = (secs * 100.0).round() as u64;

    let centis = total_centis % 100;
    let total_secs = total_centis / 100;

    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}.{:02}", hours, minutes, seconds, centis)
    } else {
        format!("{:02}:{:02}.{:02}", minutes, seconds, centis)
    }
}

// 用于 advmix 子命令
struct Track {
    player: Player,
    start: f64,
    total: f64,
    played: bool,
}

fn main() -> Result<()> {
    let args = cli().context("命令行解析错误")?.get_matches();

    let quiet_mode: &bool = args.get_one::<bool>("quiet").unwrap_or(&false);
    if let Some(moo) = args.subcommand_matches("moo") {
        let moo_level =
            (moo.get_count("verbose") as i32 - moo.get_count("quiet") as i32).clamp(-9, 9);

        match moo_level {
            -9 => {
                eprintln!(
                    "那个链接是 GTA 8 宣传片, 你可以去看看\n(字符画艺术的文本内容使用了 cowsay 来生成)"
                )
            }
            -8 => eprintln!(
                r#"骗你的其实有

 ________________________
< https://b23.tv/kh0LuNB >
 ------------------------
         \     ,-.      .-,
          \    |-.\ __ /.-|
           \   \  `    `  /
                /_     _ \
              <  _`q  p _  >
              <.._=/  \=_. >
                 {{`\()/`}}`\
                 {{      }}  \
                 |{{    }}    \
                 \ '--'   .- \
                 |-      /    \
                 | | | | |     ;
                 | | |.;.,..__ |
               .-"";`         `|
              /    |           /
              `-../____,..---'`"#
            ),
            -7 => eprintln!("... 好吧, 骗你的, 压根就没有存在于平行宇宙的彩蛋 ...?"),
            -6 => eprintln!("如果我给你一个来自于平行宇宙的彩蛋, 你能停手吗?"),
            -5 => eprintln!("我需要更多的 -v !!!"),
            -4 => eprintln!("回头! 给 -v 参数而不是 -q !"),
            -3 => eprintln!("... 你的方向是不是反了 ...?"),
            -2 => eprintln!("本程序真的有很多彩蛋 ..."),
            -1 => eprintln!("本程序有很多彩蛋"),
            0 => eprintln!("本程序没有彩蛋"),
            1 => eprintln!("本程序真的没有彩蛋"),
            2 => eprintln!("我不是说了这个程序没有彩蛋了吗?"),
            3 => eprintln!("相信你的直觉, 你在源代码看到的不一定为实"),
            4 => eprintln!(
                "嘿, 你听过有个名字叫 aptitude 的包管理器吗?\n或许这个时候我应该给你彩蛋了 ..."
            ),
            5 => eprintln!("🥚 (不支持 Emoji 的终端可能看不到)"),
            6 => eprintln!("这是什么? 一个普通的蛋, 甚至还不能称得上是 \"复活节彩蛋\""),
            7 => eprintln!("我或许可以给你点真正的彩蛋, 一个 Rust 专属的彩蛋 ..."),
            8 => {
                eprintln!("Rust 专属的彩蛋™\n");
                ferris_says::say(
                    "https://b23.tv/dpzoGKK",
                    24,
                    BufWriter::new(stderr().lock()),
                )?;
            }
            9 => eprintln!("那条链接貌似是 GTA 7 宣传片, 你可以看看"),
            _ => eprintln!("一个不可能会出现的输出, 貌似是另一种形式的彩蛋 ...?"),
        }
    } else if let Some(mix) = args.subcommand_matches("mix") {
        let mut sink = DeviceSinkBuilder::open_default_sink()?;
        sink.log_on_drop(false);

        if !quiet_mode {
            eprintln!(
                "警告: 请在音频播放完成后手动通过 Ctrl+C 退出, 本命令不关心最后播放完的音频是哪个, 因为目前没有办法获取 OGG 格式音频的长度\n"
            )
        }
        for (i, filename) in mix
            .get_many::<String>("FILE")
            .context("传入的 <FILE> 参数类型错误")?
            .enumerate()
        {
            let file = BufReader::new(
                File::open(filename).with_context(|| format!("文件不存在: {}", filename))?,
            );
            let decoder = Decoder::new(file)?;

            if !quiet_mode {
                println!(
                    "音频 #{} 开始播放, 长度: {}",
                    i,
                    match decoder.total_duration() {
                        None => String::from("?"),
                        Some(x) => format_seconds(x.as_secs_f64()),
                    }
                );
            }

            sink.mixer().add(decoder.amplify(0.1));
            sleep(Duration::from_secs_f64(
                *(mix.get_one::<f64>("time").unwrap_or(&0.0)),
            ));
        }

        if !quiet_mode {
            println!("\n所有音频均已开始播放, 按下 Ctrl+C 退出...");
        }
        let stopped = Arc::new(AtomicBool::new(false));
        {
            let stopped = Arc::clone(&stopped);
            ctrlc::set_handler(move || stopped.store(true, Ordering::Release))?;
        }
        loop {
            if stopped.load(Ordering::Acquire) {
                break;
            }
            sleep(Duration::from_millis(10));
        }
    } else if let Some(advmix) = args.subcommand_matches("advmix") {
        let mut sink = DeviceSinkBuilder::open_default_sink()?;
        sink.log_on_drop(false);

        let speed = *advmix.get_one::<f32>("speed").unwrap_or(&1.0);
        let volume = *advmix.get_one::<f64>("volume").unwrap_or(&1.0);
        let interval = *advmix.get_one::<f64>("time").unwrap_or(&0.0);

        let mut tracks: Vec<Track> = Vec::new();

        // 预加载，因为我觉得播放时加载很麻烦 (
        for (i, filename) in advmix
            .get_many::<String>("FILE")
            .context("传入的 <FILE> 参数类型错误")?
            .enumerate()
        {
            let file = BufReader::new(
                File::open(filename).with_context(|| format!("文件不存在: {}", filename))?,
            );
            let decoder = Decoder::new(file)?;
            let total = match decoder.total_duration() {
                Some(x) => x.as_secs_f64(),
                None => -1.0,
            };
            let player = Player::connect_new(sink.mixer());
            player.pause();
            player.set_volume(volume);
            player.set_speed(speed);
            player.append(decoder);

            tracks.push(Track {
                player,
                start: (i as f64) * interval,
                total,
                played: false,
            });
        }

        let time_start = Instant::now();
        let mut out: Vec<String> = vec![String::new(); tracks.len()];
        let mut max_out_len: Vec<usize> = vec![0; tracks.len()];

        let mut stop_afterupdate=false;
        let stop_ctrlc = Arc::new(AtomicBool::new(false));
        {
            let stop_ctrlc = Arc::clone(&stop_ctrlc);
            ctrlc::set_handler(move || {
                stop_ctrlc.store(true, Ordering::Release);
            })?;
        }

        if !quiet_mode {
            print!("{}", "\n".repeat(tracks.len()));
        }
        loop {
            if stop_ctrlc.load(Ordering::Acquire) {
                println!("\n退出...");
                tracks.iter().for_each(|x| x.player.stop());
                break;
            }

            let time_passed = time_start.elapsed().as_secs_f64();

            let mut played_count = 0;
            for (i, track) in tracks.iter_mut().enumerate() {
                if !quiet_mode {
                    if track.player.is_paused() || track.player.empty() {
                        out[i] = format!(
                            "[{}] 长度: {}",
                            if track.player.is_paused() {
                                "WAIT"
                            } else {
                                track.played = true;
                                "DONE"
                            },
                            format_seconds(track.total)
                        );
                    } else {
                        out[i] = format!(
                            "[PLAY] 位置: {} / {}",
                            format_seconds(track.player.get_pos().as_secs_f64() * speed as f64),
                            format_seconds(track.total)
                        );
                    }

                    let out_len = out[i].len();
                    max_out_len[i] = max(max_out_len[i], out_len);
                    out[i] += &" ".repeat(max_out_len[i] - out_len);
                }

                if time_passed * speed as f64 > track.start {
                    track.player.play();
                }

                if track.played {
                    played_count += 1;
                }
            }

            if played_count == tracks.len() {
                if stop_afterupdate {
                    break;
                } else {
                    stop_afterupdate=true;
                }
            }

            if !quiet_mode {
                execute!(
                    stdout(),
                    MoveUp(tracks.len() as u16),
                    Print(out.join("\n") + "\n")
                )?;
            }

            sleep(Duration::from_millis(10));
        }
    } else {
        let mut sink = DeviceSinkBuilder::open_default_sink()?;
        sink.log_on_drop(false);
        let filename = args
            .get_one::<String>("FILE")
            .context("传入的 <FILE> 参数类型错误")?;

        let file = BufReader::new(
            File::open(filename).with_context(|| format!("文件不存在: {}", filename))?,
        );
        let decoder = Decoder::new(file)?;
        let total_duration = match decoder.total_duration() {
            None => -1.0,
            Some(x) => x.as_secs_f64(),
        };
        let player = Player::connect_new(sink.mixer());

        player.set_volume(*args.get_one::<f64>("volume").unwrap_or(&1.0));
        player.set_speed(*args.get_one::<f32>("speed").unwrap_or(&1.0));
        player.append(decoder);

        let mut max_output_len = 0;
        if !quiet_mode {
            println!();
        }
        let stopped = Arc::new(AtomicBool::new(false));
        {
            let stopped = Arc::clone(&stopped);
            ctrlc::set_handler(move || {
                stopped.store(true, Ordering::Release);
            })?;
        }
        loop {
            if stopped.load(Ordering::Acquire) {
                println!("\n退出...");
                player.stop();
                break;
            }
            if !quiet_mode {
                let output_str = format!(
                    "位置: {} / {}",
                    format_seconds(
                        player.get_pos().as_secs_f64()
                            * (*args.get_one::<f32>("speed").unwrap_or(&1.0)) as f64
                    ),
                    format_seconds(total_duration),
                );
                max_output_len = max(output_str.len(), max_output_len);
                execute!(
                    stdout(),
                    MoveUp(1),
                    Print(
                        format!(
                            "{}{}",
                            output_str,
                            " ".repeat(max_output_len - output_str.len())
                        ) + "\n"
                    )
                )?;
            }
            if player.empty() {
                break;
            }
            sleep(Duration::from_millis(10));
        }
    }

    Ok(())
}
