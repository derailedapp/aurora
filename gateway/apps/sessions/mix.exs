defmodule Derailed.Session.MixProject do
  use Mix.Project

  def project do
    [
      app: :sessions,
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
      mod: {Derailed.Session.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:db, in_umbrella: true},
      {:guilds, in_umbrella: true},
      {:gen_registry, "~> 1.3.0"},
      {:postgrex, "~> 0.19"},
      {:manifold, "~> 1.0"},
      {:jason, "~> 1.4"}
    ]
  end
end
