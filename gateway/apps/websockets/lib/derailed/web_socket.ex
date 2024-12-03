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

  import Dotenvy

  defp op_to_atom(t) do
    %{
      "0": :ready
    }[t]
  end

  def init(req, _state) do
    value = :cowboy_req.header("authorization", req, :undefined)

    if value == :undefined do
      {:ok, req, %{header: nil}}
    else
      {:cowboy_websocket, req,
       %{
         header: value
       }}
    end
  end

  def websocket_init(state) do
    source!([".env", System.get_env()])

    if state[:header] == nil do
      {[:close, 4001, "Invalid authorization token"], %{}}
    else
      {[],
       %{
         sequence: 0,
         ready: false,
         session_id: nil,
         session_pid: nil,
         session_ref: nil
       }}
    end
  end

  def websocket_handle({:text, raw_data}, state) do
    {:ok, data} = Jsonrs.decode(raw_data)

    d = Map.new(data)
    handle(op_to_atom(d[:op]), d[:d], state)
  end

  def websocket_handle(_any, state) do
    {[], state}
  end

  def handle(_type, _data, state) do
    {[:close, 4001, "Invalid message type"], state}
  end

  def websocket_info({:event, type, data}, state) do
    {[:text, Jsonrs.encode!(%{t: type, d: data, s: state[:sequence] + 1})],
     %{state | sequence: state[:sequence] + 1}}
  end

  def websocket_info(_any, state) do
    {[], state}
  end
end
