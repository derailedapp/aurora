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

defmodule Derailed.WebSocket.Cowboy do
  def get_dispatch do
    :cowboy_router.compile([
      {:_,
       [
         {"/", Derailed.WebSocket, %{}}
       ]}
    ])
  end

  def start_link do
    {:ok, _} =
      :cowboy.start_clear(
        :derailed,
        [{:port, 16600}],
        %{
          env: %{
            dispatch: get_dispatch()
          }
        }
      )
  end
end
