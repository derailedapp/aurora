defmodule Derailed.DBTest do
  use ExUnit.Case
  doctest Derailed.DB

  test "greets the world" do
    assert Derailed.DB.hello() == :world
  end
end
