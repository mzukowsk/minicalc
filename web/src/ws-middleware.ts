import { MiddlewareAPI, Dispatch, AnyAction } from 'redux';
import { updateCells, clearAll } from './cells-slice';
import { ConnectionStatus, setStatus } from './connection-slice';

const WEBSOCKET_CONNECT = 'websocket/connect';
const WEBSOCKET_UPDATE_CELL = 'websocket/updateCell';

type CellUpdateRequest = {
  col: number,
  row: number,
  expression: string | null,
}

type CellUpdateResponse = {
  col: number,
  row: number,
  value: string | null,
  error: string | null,
}

interface WebsocketCellUpdateAction {
  type: string;
  payload: CellUpdateRequest;
}

export const websocketCellUpdate = (col: number, row: number, expression: string | null) => {
  const action: WebsocketCellUpdateAction = {
    type: WEBSOCKET_UPDATE_CELL,
    payload: {
      col,
      row,
      expression
    }
  }
  return action;
}

export const connect = () => {
  const action: AnyAction = {
    type: WEBSOCKET_CONNECT
  }
  return action;
}

export const createWebSocketMiddleware = (url: string) => {
  return (storeAPI: MiddlewareAPI) => {
    let socket: WebSocket | null = null;

    return (next: Dispatch<AnyAction>) => (action: AnyAction) => {
      if (action.type === WEBSOCKET_CONNECT) {
        socket = new WebSocket(url);

        socket.onmessage = (ev: MessageEvent<string>) => {
          const data: CellUpdateResponse[] = JSON.parse(ev.data);
          storeAPI.dispatch(updateCells(data.map(r => ({ x: r.col, y: r.row, value: r.value, error: r.error }))));
        };

        socket.onclose = () => {
          storeAPI.dispatch(setStatus({ status: ConnectionStatus.Disconnected, statusText: `Failed to connect to ${url}`}));
          socket = null;
        };

        socket.onopen = () => {
          storeAPI.dispatch(setStatus({ status: ConnectionStatus.Connected, statusText: `Connected to ${url}`}));
          storeAPI.dispatch(clearAll());
        };

        storeAPI.dispatch(setStatus({ status: ConnectionStatus.Connecting, statusText: `Connecting to ${url}`}));
        return next(action);
      }

      if (action.type === WEBSOCKET_UPDATE_CELL) {
        if (!socket)
          return next(action);
        socket.send(JSON.stringify(action.payload));
        return next(action);
      }

      return next(action);
    }
  }
}
