import { createAsyncThunk, createSlice, createEntityAdapter, PayloadAction, CaseReducer } from '@reduxjs/toolkit';
import { websocketCellUpdate } from './ws-middleware';

// store

type CellValue = {
  x: number;
  y: number;
  expression: string;
  value: string | null;
  error: string | null;
}

const getCellId = (x: number, y: number) => x.toString() + ':' + y.toString();

export const cellsAdapter = createEntityAdapter<CellValue>({
  selectId: (cell) => getCellId(cell.x, cell.y),
  sortComparer: (a, b) => (a.x - b.x) || (a.y - b.y),
});

type EditedCell = {
  x: number,
  y: number
} | null;

// By default, `createEntityAdapter` gives you `{ ids: [], entities: {} }`.
// If you want to track 'loading' or other keys, you would initialize them here:
// `getInitialState({ loading: false, activeRequestId: null })`
const initialState = cellsAdapter.getInitialState({ editedCell: null as EditedCell});

type StoreType = typeof initialState;


// actions

type UpdateCellParams = {
  x: number,
  y: number,
  expression: string | null,
}

export const updateCell = createAsyncThunk('cells/updateCell', async (arg: UpdateCellParams, { dispatch }) => {
  dispatch(websocketCellUpdate(arg.x, arg.y, arg.expression));
  return arg;
});

const updateCellReducer: CaseReducer<StoreType, PayloadAction<UpdateCellParams>> = (state, { payload }) => {
  if (payload.expression)
    cellsAdapter.upsertOne(state, { x: payload.x, y: payload.y, expression: payload.expression, value: null, error: null });
  else
    cellsAdapter.removeOne(state, getCellId(payload.x, payload.y));
};

type UpdateCellsParams = {
  x: number,
  y: number,
  value: string | null,
  error: string | null,
}

// const updateCells: CaseReducer<StoreType, PayloadAction<CellValue[]>> = (state, { payload }) => {
//   usersAdapter.upsertMany(state, payload);
// };

export const slice = createSlice({
  name: 'cells',
  initialState,
  reducers: {
    setEditedCell: (state, action: PayloadAction<EditedCell>) => {
      state.editedCell = action.payload;
    },
    updateCells: (state, action: PayloadAction<UpdateCellsParams[]>) => {
      const updates = action.payload.map(p => ({ id: getCellId(p.x, p.y), changes: { value: p.value, error: p.error }}));
      cellsAdapter.updateMany(state, updates);
    },
    clearAll: (state) => {
      cellsAdapter.removeAll(state);
    }
  },
  extraReducers: (builder) => {
    builder.addCase(updateCell.fulfilled, updateCellReducer);
  },
});

export default slice.reducer;

export const selectCellValue = (x: number, y: number) => (state: StoreType) => state.entities[getCellId(x, y)];

export const { setEditedCell, updateCells, clearAll } = slice.actions;

export const selectCellEdited = (x: number, y: number) => (state: StoreType) => state.editedCell !== null && state.editedCell.x === x && state.editedCell.y === y;
