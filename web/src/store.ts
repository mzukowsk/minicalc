import { TypedUseSelectorHook, useDispatch, useSelector } from 'react-redux';
import { combineReducers, configureStore } from '@reduxjs/toolkit';
import cellReducer from './cells-slice';
import connectionReducer from './connection-slice';
import modalReducer from './modal-slice';
import { createWebSocketMiddleware } from './ws-middleware';

const rootReducer = combineReducers({
  cells: cellReducer,
  connection: connectionReducer,
  modal: modalReducer,
});

export const store = configureStore({
  reducer: rootReducer,
  middleware: (getDefaultMiddleware) => getDefaultMiddleware().concat(createWebSocketMiddleware('ws://127.0.0.1:9123'))
});

export type AppDispatch = typeof store.dispatch;
export type RootState = ReturnType<typeof store.getState>;
export const useAppDispatch: () => AppDispatch = useDispatch;
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;
