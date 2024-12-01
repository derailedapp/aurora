import gleam/erlang/process
import gleam/otp/actor

pub type Message(element) {
  Shutdown
}

pub type SessionState {
  SessionState(ws_pid: process.Pid)
}

pub fn handle_message(
  message: Message(e),
  _state,
) -> actor.Next(Message(e), List(e)) {
  case message {
    Shutdown -> {
      actor.Stop(process.Normal)
    }
  }
}
