import dot_env as dot
import dot_env/env
import gleam/bytes_tree
import gleam/dict.{type Dict}
import gleam/erlang/atom.{type Atom}
import gleam/erlang/process
import gleam/http/request.{type Request}
import gleam/http/response.{type Response}
import gleam/option.{Some}
import gleam/otp/actor
import gleam/string
import logging
import mist.{type Connection, type ResponseData}

@external(erlang, "logger", "update_primary_config")
fn logger_update_primary_config(config: Dict(Atom, Atom)) -> Result(Nil, any)

pub fn main() {
  logging.configure()

  let _ =
    logger_update_primary_config(
      dict.from_list([
        #(atom.create_from_string("level"), atom.create_from_string("debug")),
      ]),
    )

  dot.load_default()

  let selector = process.new_selector()
  let state = Nil

  let not_found =
    response.new(404)
    |> response.set_body(mist.Bytes(bytes_tree.new()))

  let assert Ok(_) =
    fn(req: Request(Connection)) -> Response(ResponseData) {
      logging.log(
        logging.Info,
        "Got a request from: " <> string.inspect(mist.get_client_info(req.body)),
      )

      case request.path_segments(req) {
        [] ->
          mist.websocket(
            request: req,
            on_init: fn(_conn) { #(state, Some(selector)) },
            on_close: fn(_state) { Nil },
            handler: handle_ws_message,
          )
        [single] -> {
          let var = env.get_string_or("GATEWAY_COM_SECRET", "SUPER_SECRET")
          case single == var {
            True -> todo
            False -> not_found
          }
        }
        _ -> not_found
      }
    }
    |> mist.new
    |> mist.bind("localhost")
    |> mist.with_ipv6
    |> mist.port(0)
    |> mist.start_http

  process.sleep_forever()
}

fn handle_ws_message(state, conn, message) {
  case message {
    mist.Text(json) -> {
      actor.continue(state)
    }
    mist.Text(_) | mist.Binary(_) -> {
      actor.continue(state)
    }
    mist.Closed | mist.Shutdown -> actor.Stop(process.Normal)
    _ -> actor.continue(state)
  }
}
