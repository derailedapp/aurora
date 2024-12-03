defmodule Derailed.SessionTest do
  use ExUnit.Case
  doctest Derailed.Session

  test "greets the world" do
    assert Derailed.Session.hello() == :world
  end
end
