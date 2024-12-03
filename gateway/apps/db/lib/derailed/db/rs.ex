defmodule Derailed.DB.Rs do
  use Rustler, otp_app: :db, crate: "derailed_db"

  # When your NIF is loaded, it will override this function.
  @spec get_token_session_id(String.t()) :: {:ok, String.t()} | {:error, :invalid_token}
  def get_token_session_id(_token), do: :erlang.nif_error(:nif_not_loaded)

  @spec get_chronological_id() :: String.t()
  def get_chronological_id(), do: :erlang.nif_error(:nif_not_loaded)
end
