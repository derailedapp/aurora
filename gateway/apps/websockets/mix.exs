defmodule Derailed.WebSocket.MixProject do
  use Mix.Project

  def project do
    [
      app: :websockets,
      version: "0.1.0",
      build_path: "../../_build",
      config_path: "../../config/config.exs",
      deps_path: "../../deps",
      lockfile: "../../mix.lock",
      elixir: "~> 1.17",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger],
      mod: {Derailed.WebSocket.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:cowboy, "~> 2.12"},
      {:jsonrs, github: "derailedapp/jsonrs"},
      {:dotenvy, "~> 0.9"},
      {:postgrex, "~> 0.19"},
      {:nanoid, "~> 2.1"},
      {:sessions, in_umbrella: true}
    ]
  end
end
