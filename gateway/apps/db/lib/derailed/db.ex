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

defmodule Derailed.DB do
  @spec struct_to_map(struct()) :: map()
  def struct_to_map(struct) do
    m = Map.from_struct(struct)
    Map.delete(m, :__meta__)
  end

  def map(result) do
    case result do
      %{rows: nil} ->
        {:error, :no_rows_nil}

      %{rows: []} ->
        {:error, :no_rows_empty}

      %{rows: [row], columns: columns} ->
        {:ok, mapify(columns, row)}

      _ ->
        {:error, :mapping_error}
    end
  end

  def maps(results) do
    case results do
      %{rows: nil} ->
        {:error, :no_rows_nil}

      %{rows: []} ->
        {:ok, []}

      %{rows: rows, columns: columns} ->
        {:ok, Enum.map(rows, fn row -> mapify(columns, row) end)}

      _ ->
        {:error, :mapping_error}
    end
  end

  def mappy(map) do
    Map.new(
      Enum.map(map, fn {k, v} ->
        {k, valuem(v)}
      end)
    )
  end

  defp mapify(columns, row) do
    val =
      columns
      |> Enum.zip(row)
      |> Map.new()

    mappy(val)
  end

  defp valuem(v) do
    cond do
      is_struct(v) ->
        struct_to_map(v)

      is_map(v) ->
        mappy(v)

      is_list(v) ->
        Enum.map(v, fn vv -> valuem(vv) end)

      true ->
        v
    end
  end
end
