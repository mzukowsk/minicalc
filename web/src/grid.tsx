import Table from 'react-bootstrap/Table';
import Cell from './cell';

const rows = 20;
const cols = 20;

export default function Grid() {
  return (
    <Table responsive striped bordered size='sm'>
      <thead>
        <tr>
          <th>#</th>
          {Array.from({ length: cols }).map((_, index) => (
              <th key={index}>{String.fromCharCode(65 + index)}</th>
          ))}
        </tr>
      </thead>
      <tbody>
        {Array.from({ length: rows }).map((_, rindex) => (
          <tr key={rindex}>
            <th>{rindex + 1}</th>
            {Array.from({ length: cols }).map((_, cindex) => (
              <Cell key={cindex} x={cindex} y={rindex} />
            ))}
          </tr>
        ))}
      </tbody>
    </Table>
  )
}
