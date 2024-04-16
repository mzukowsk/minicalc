import { useEffect, useRef, useState } from 'preact/hooks';
import Stack from 'react-bootstrap/Stack';
import { selectCellEdited, selectCellValue, setEditedCell, updateCell } from './cells-slice';
import { useAppDispatch, useAppSelector } from './store';
import { ComponentChildren } from 'preact';
import { ConnectionStatus } from './connection-slice';

type CellProps = {
  x: number;
  y: number;
}

export default function Cell({x, y}: CellProps) {
  const cellValue = useAppSelector((state) => selectCellValue(x, y)(state.cells));
  const dispatch = useAppDispatch();
  const isEdited = useAppSelector((state) => selectCellEdited(x, y)(state.cells));
  const setEdited = (edited: boolean) => dispatch(setEditedCell(edited ? {x, y} : null));
  const status = useAppSelector((state) => state.connection.status);

  const isEditable = status === ConnectionStatus.Connected;
  
  let control: ComponentChildren;
  
  if (isEdited) {
    const inputRef = useRef(null);
    const [expression, setExpression] = useState(cellValue?.expression || '');

    const finishEditing = async (confirmChanges: boolean) => {
      let trimmed_expression = expression.trimEnd();
      if (trimmed_expression.valueOf() != (cellValue?.expression || '').valueOf()) {
        if (confirmChanges)
          await dispatch(updateCell({x, y, expression: trimmed_expression || null}));
        else
          setExpression(cellValue?.expression || '');
      }
      setEdited(false);
    }

    useEffect(() => {
      if (isEdited) {
        if (inputRef && inputRef.current) {
          (inputRef.current as HTMLInputElement).focus();
        }
      } else {
        finishEditing(true);
      }
    }, [isEdited, inputRef]);

    const onKeyDown = async (key: string) => {
      if (key === 'Enter' || key === 'NumpadEnter' || key === 'Tab') {
        await finishEditing(true);
      } else if (key === 'Escape') {
        await finishEditing(false);
      }
    }
 
    control = <input
                ref={inputRef}
                type="text"
                name="cell-edit"
                className="form-control form-control-sm"
                placeholder="Enter formula..."
                value={expression}
                onInput={e => setExpression((e.target as HTMLInputElement).value)}
                onKeyDown={async e => await onKeyDown(e.code)}
                onBlur={async () => await finishEditing(true)}
              />
  } else if (cellValue?.error) {
    control = <div className='container'>{cellValue.expression}</div>
  } else {
    control = <div className='container'>{cellValue?.value}</div>
  }

  if (cellValue?.error)
  {
    control = <Stack>{control}<pre className='text-start small mb-0'><small>{cellValue?.error}</small></pre></Stack>
  }
  
  return (
    <td className={(cellValue?.error || undefined) && 'table-danger'} onClick={() => isEditable && !isEdited && setEdited(true)}>
      {control}
    </td>
  )
}
