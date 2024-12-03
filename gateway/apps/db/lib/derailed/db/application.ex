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

defmodule Derailed.DB.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application
  import Dotenvy

  @impl true
  def start(_type, _args) do
    source!(["../.env", ".env", System.get_env()])

    uri = parse(env!("DATABASE_URL"))
    uri = Keyword.put(uri, :name, :db)

    children = [
      {Postgrex, uri}
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: Derailed.DB.Supervisor]
    Supervisor.start_link(children, opts)
  end

  defp parse(connurl) do
    uri = URI.parse(connurl)
    "postgres" = uri.scheme

    uri_params =
      Enum.filter(Map.to_list(uri), fn {k, _v} ->
        k in MapSet.new([:host, :port, :path, :userinfo])
      end)

    query_params =
      case uri.query do
        nil -> []
        _ -> URI.decode_query(uri.query)
      end

    (Enum.map(uri_params, &uri_param_map/1) ++ Enum.map(query_params, &query_param_map/1))
    |> Enum.filter(fn thing -> thing != nil end)
    |> List.flatten()
    |> Keyword.new()
  end

  defp uri_param_map(kvpair) do
    case kvpair do
      {_, nil} -> nil
      {:host, ""} -> []
      {:host, v} -> [hostname: v]
      {:port, v} -> [port: v]
      {:path, v} -> [database: String.replace_prefix(v, "/", "")]
      {:userinfo, v} -> [List.zip([[:username, :password], String.split(v, ":", parts: 2)])]
    end
  end

  defp query_param_map(kvpair) do
    case kvpair do
      {_, ""} ->
        []

      {"port", v} ->
        [port: String.to_integer(v)]

      {"host", v} ->
        case String.starts_with?(v, "/") do
          true -> [socket_dir: v]
          _ -> [hostname: v]
        end

      {"user", v} ->
        [username: v]

      {"dbname", v} ->
        [database: v]

      {"sslmode", "disable"} ->
        [ssl: false]

      {"sslmode", _} ->
        [ssl: true]

      {"connect_timeout", v} ->
        [connect_timeout: String.to_integer(v) * 1000]

      {somek, somev} ->
        Keyword.new([{String.to_atom(somek), somev}])
    end
  end
end
