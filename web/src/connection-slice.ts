import { PayloadAction, createSlice } from '@reduxjs/toolkit';

export enum ConnectionStatus {
  Disconnected,
  Connecting,
  Connected,
}

interface ConnectionState {
  status: ConnectionStatus;
  statusText: string;
}

const initialState: ConnectionState = {
  status: ConnectionStatus.Disconnected,
  statusText: 'Disconnected'
};

type SetStatusParams = {
  status: ConnectionStatus,
  statusText: string,
};

const modalSlice = createSlice({
  name: 'connection',
  initialState,
  reducers: {
    setStatus: (state, action: PayloadAction<SetStatusParams>) => {
      state.status = action.payload.status;
      state.statusText = action.payload.statusText;
    },
  },
});

export const { setStatus } = modalSlice.actions;

export default modalSlice.reducer;
