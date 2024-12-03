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

defmodule Derailed.Guild do
  use GenServer

  def start_link(d) do
    GenServer.start_link(__MODULE__, d)
  end

  def init({guild_id, data}) do
    guild_data =
      if not is_nil(data) do
        data
      else
        {_, result} =
          Postgrex.prepare_execute!(
            :db,
            "get_guild_genserver",
            "SELECT * FROM guilds WHERE id = $1;",
            [guild_id]
          )

        Derailed.DB.map(result)
      end

    # TODO: if the guild is a foreign guild, fetch via HTTP instead.
    {_, result} =
      Postgrex.prepare_execute!(
        :db,
        "get_guild_members_genserver",
        "SELECT * FROM guild_members WHERE guild_id = $1;",
        [guild_id]
      )

    members = Derailed.DB.map(result)

    {:ok,
     %{
       guild_data: guild_data,
       member_data: members,
       session_pids: MapSet.new(),
       session_refs: MapSet.new()
     }}
  end

  @spec subscribe(pid(), pid()) :: :ok
  def subscribe(pid, session_pid) do
    GenServer.call(pid, {:subscribe, session_pid})
  end

  @spec send(pid(), nonempty_charlist(), pid()) :: :ok
  def send(pid, type, data) do
    GenServer.call(pid, {:send, type, data})
  end

  # TODO: this quite obviously doesn't scale well on big guilds.
  @spec get_members(pid()) :: list()
  def get_members(pid) do
    GenServer.call(pid, :get_members)
  end

  def handle_call({:subscribe, session_pid}, _from, state) do
    {:reply, :ok,
     %{
       state
       | session_pids: MapSet.put(state[:session_pids], session_pid),
         session_refs: MapSet.put(state[:session_refs], Process.monitor(session_pid))
     }}
  end

  def handle_call({:send, type, data}, _from, state) do
    Manifold.send(state[:session_pids], {:event, :channel, type, data})
    {:reply, :ok, state}
  end

  def handle_call(:get_members, _from, state) do
    {:reply, state[:member_data], state}
  end

  # TODO: explore ZenMonitor
  # TODO: handle distribution, `pid` would be `{name, node}`
  def handle_info({:DOWN, ref, :process, pid, _reason}, state) do
    m = MapSet.delete(state.session_pids, pid)
    if Enum.empty?(m) do
      {:stop, :no_subscribers, state}
    else
      {:noreply, %{state | session_pids: m, session_refs: MapSet.delete(state.session_refs, ref)}}
    end
  end
end
