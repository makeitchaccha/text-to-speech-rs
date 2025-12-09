use std::fmt::{Display, Formatter};
use anyhow::anyhow;
use crate::command::{Context, Result};

#[poise::command(slash_command, guild_only, subcommands("user", "guild"), subcommand_required)]
pub async fn voice(_: Context<'_>) -> Result<()> {
    Ok(())
}

#[poise::command(slash_command, subcommands("choose", "clear"), subcommand_required)]
pub async fn user(ctx: Context<'_>) -> Result<()> {
    Ok(())
}

#[poise::command(slash_command, subcommands("choose", "clear"), subcommand_required)]
pub async fn guild(ctx: Context<'_>) -> Result<()> {
    Ok(())
}

enum Scope {
    User,
    Guild,
}

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::User => write!(f, "user"),
            Scope::Guild => write!(f, "guild"),
        }
    }
}

fn find_scope(ctx: Context<'_>) -> Result<Scope> {
    let scope = ctx.parent_commands().last().ok_or(anyhow!("missing parent"))?;
    match scope.name.as_str() {
        "user" => Ok(Scope::User),
        "guild" => Ok(Scope::Guild),
        _ => anyhow::bail!("unknown option")
    }
}

#[poise::command(slash_command)]
pub async fn choose(
    ctx: Context<'_>,
    #[description = "Name of voice to choose"]
    name: String,
) -> Result<()> {
    let scope = find_scope(ctx)?;

    if ctx.data().registry.get(name.as_str()).is_none() {
        return Err(anyhow::anyhow!(format!("voice {} not found", name)));
    }

    match scope {
        Scope::User => {
            ctx.data().repository.save_user(ctx.author().id, &name).await?;
        },
        Scope::Guild => {
            ctx.data().repository.save_guild(ctx.guild_id().ok_or(anyhow!("guild not found"))?, &name).await?;
        }
    }

    ctx.say(format!("{} was chosen for {} voice.", name, scope)).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn clear(
    ctx: Context<'_>
) -> Result<()> {
    let scope = find_scope(ctx)?;

    match scope {
        Scope::User => {
            ctx.data().repository.delete_user(ctx.author().id).await?;
        },
        Scope::Guild => {
            ctx.data().repository.delete_guild(ctx.guild_id().ok_or(anyhow!("guild not found"))?).await?;
        }
    }

    ctx.say(format!("voice was cleared for {} voice.", scope)).await?;

    Ok(())
}
