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

defmodule Derailed.DB.Rs do
  use Rustler, otp_app: :db, crate: "derailed_db"

  # When your NIF is loaded, it will override this function.
  @spec get_token_session_id(String.t()) :: {:ok, String.t()} | {:error, :invalid_token}
  def get_token_session_id(_token), do: :erlang.nif_error(:nif_not_loaded)

  @spec get_chronological_id() :: String.t()
  def get_chronological_id(), do: :erlang.nif_error(:nif_not_loaded)
end
