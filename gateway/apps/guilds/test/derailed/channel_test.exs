defmodule Derailed.GuildTest do
  use ExUnit.Case
  doctest Derailed.Guild

  test "greets the world" do
    assert Derailed.Guild.hello() == :world
  end
end
