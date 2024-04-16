import { createSlice } from '@reduxjs/toolkit';
import { AppDispatch } from './store';

export type ModalButton = {
  variant?: string;
  label: string;
  callback?: () => void;
};

export type ModalData = {
  title?: React.ReactNode;
  body: React.ReactNode;
  buttons?: ModalButton[];
};

interface ModalState {
  modalData: ModalData | null;
}

const initialState: ModalState = {
  modalData: null,
};

const modalSlice = createSlice({
  name: 'modal',
  initialState,
  reducers: {
    setModalData: (state, action) => {
      state.modalData = action.payload;
    },
  },
});

export const { setModalData } = modalSlice.actions;

export default modalSlice.reducer;

export const useShowModal = (dispatch: AppDispatch) => (modal?: ModalData) => dispatch(setModalData(modal || null));
