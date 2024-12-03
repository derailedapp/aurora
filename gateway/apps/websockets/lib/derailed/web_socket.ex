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
