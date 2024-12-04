# Copyright (C) 2024 V.J. De Chico
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published
# by the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

defmodule Derailed.WebSocket do
  @behaviour :cowboy_websocket

  defp op_to_atom(t) do
    %{
      0 => :identify
      # 1 => :dispatch,
      # 2 => :hello
    }[t]
  end

  def init(req, _state) do
    {:cowboy_websocket, req, %{}, %{"compress" => true}}
  end

  def websocket_init(_state) do
    # TODO: websocket hb timeout
    {[
       {:text,
        Jason.encode!(%{
          op: 2,
          d: nil
        })}
     ],
     %{
       sequence: 0,
       ready: false,
       session_id: nil,
       session_pid: nil,
       session_ref: nil
     }}
  end

  def websocket_handle({:text, raw_data}, state) do
    {:ok, data} = Jason.decode(raw_data)

    d = Map.new(data)
    handle(op_to_atom(Map.get(d, "op")), Map.get(d, "d"), state)
  end

  def websocket_handle(_any, state) do
    {[], state}
  end

  def handle(:identify, data, state) do
    case Derailed.Contracts.Identify.conform(data) do
      {:ok, model} ->
        case Derailed.DB.Rs.get_token_session_id(Map.get(model, "token")) do
          {:ok, session_id} ->
            {_, result} =
              Postgrex.prepare_execute!(
                :db,
                "get_user_from_session_id",
                "SELECT user_id FROM sessions WHERE id = $1;",
                [session_id]
              )

            {:ok, user_id_r} = Derailed.DB.map(result)

            user_id = Map.get(user_id_r, "user_id")

            {:ok, session_pid} =
              GenRegistry.start(Derailed.Session, session_id, [{session_id, user_id, self()}])

            session_ref = Process.monitor(session_pid)

            Derailed.Session.send_ready(session_pid)

            {[],
             %{
               state
               | session_pid: session_pid,
                 session_ref: session_ref,
                 session_id: session_id,
                 ready: true
             }}

          {:error, :invalid_token} ->
            {[{:close, 4003, "Invalid token"}], state}
        end

      {:error, why} ->
        {[{:close, 4002, Jason.encode!(Enum.map(why, &to_string/1))}], state}
    end
  end

  def handle(_type, _data, state) do
    {[{:close, 4001, "Invalid message type"}], state}
  end

  def websocket_info({:event, type, data}, state) do
    {[{:text, Jason.encode!(%{op: 1, t: type, d: data, s: state[:sequence] + 1})}],
     %{state | sequence: state[:sequence] + 1}}
  end

  def websocket_info({:DOWN, _ref, :process, _pid, _reason}, state) do
    {[{:close, 4004, "Internal Server Error"}], state}
  end

  def websocket_info(_any, state) do
    {[], state}
  end
end
