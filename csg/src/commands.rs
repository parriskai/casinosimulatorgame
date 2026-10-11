use std::sync::mpsc::{Receiver, Sender, TryIter, channel};
use crate::prelude::*;

pub enum GameCommand{
    Quit
}

pub struct GameCommandReciver{
    recv: Receiver<GameCommand>
} impl GameCommandReciver{
    pub fn iter(&self) -> TryIter<GameCommand>{
        self.recv.try_iter()
    }
}

pub enum RendererCommand{
    SetFrameBufferSize(WindowSize)
}

pub struct RendererCommandReciver{
    recv: Receiver<RendererCommand>
} impl RendererCommandReciver{
    pub fn iter(&self) -> TryIter<RendererCommand>{
        self.recv.try_iter()
    }
}

#[derive(Clone)]
pub struct CommandBrodcaster{
    game_send: Sender<GameCommand>,
    renderer_send: Sender<RendererCommand>
} impl CommandBrodcaster{
    pub fn raw_game_command(&self, cmd: GameCommand) -> GResult<()>{
        self.game_send.send(cmd).g_err()
    }

    pub fn quit_game(&self) -> GResult<()>{
        self.raw_game_command(GameCommand::Quit)
    }

    pub fn raw_renderer_command(&self, cmd: RendererCommand) -> GResult<()>{
        self.renderer_send.send(cmd).g_err()
    }

    pub fn set_framebuffer_size(&self, size: WindowSize) -> GResult<()>{
        self.raw_renderer_command(RendererCommand::SetFrameBufferSize(size))
    }
}

pub struct CommandRecivers{
    pub game_recv: GameCommandReciver,
    pub renderer_recv: RendererCommandReciver,
}

pub fn create_command_pair() -> (CommandBrodcaster, CommandRecivers){
    let (game_send, game_recv_inner) = channel();
    let game_recv = GameCommandReciver{recv: game_recv_inner};

    let (renderer_send, renderer_recv_inner) = channel();
    let renderer_recv = RendererCommandReciver{recv: renderer_recv_inner};

    (
        CommandBrodcaster{
            game_send,
            renderer_send,
        },
        CommandRecivers{
            game_recv,
            renderer_recv
        }
    )
}