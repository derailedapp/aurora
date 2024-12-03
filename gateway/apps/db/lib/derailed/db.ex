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
