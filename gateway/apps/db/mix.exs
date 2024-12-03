defmodule Derailed.DB.MixProject do
  use Mix.Project

  def project do
    [
      app: :db,
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
      mod: {Derailed.DB.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:postgrex, "~> 0.19"},
      {:dotenvy, "~> 0.9"},
      {:rustler, "~> 0.35.0"}
    ]
  end
end
