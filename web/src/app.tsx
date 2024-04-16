import Container from 'react-bootstrap/Container';
import Stack from 'react-bootstrap/Stack';
import Header from './header';
import Grid from './grid';
import './app.css';
import { useAppDispatch } from './store';
import { useEffect } from 'preact/hooks';
import { connect } from './ws-middleware';

export function App() {
  const dispatch = useAppDispatch();

  useEffect(() => {
    dispatch(connect());
    console.log('connect');
  }, []);

  return (
    <Container fluid>
      <Stack>
        <Header />
        <Grid />
      </Stack>
    </Container>
  );
}
