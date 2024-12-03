defmodule Derailed.WebSocket.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      %{
        id: Derailed.WebSocket.Cowboy,
        start: {Derailed.WebSocket.Cowboy, :start_link, []}
      }
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: Derailed.WebSocket.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
