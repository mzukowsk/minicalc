import Alert from 'react-bootstrap/Alert';
import Button from 'react-bootstrap/Button';
import { useAppDispatch, useAppSelector } from './store';
//import { useShowModal } from './modal-slice';
import { Col, Container, Row } from 'react-bootstrap';
import { connect } from './ws-middleware';
import { ConnectionStatus } from './connection-slice';

export default function Header() {
  const status = useAppSelector((state) => state.connection.status);
  const statusText = useAppSelector((state) => state.connection.statusText);
  const dispatch = useAppDispatch();

  // const showModal = useShowModal(dispatch);

  // const showSampleModal = () => {
  //   showModal({title: 'Sample modal', body: 'Body text'});
  // }

  const retryConnection = () => dispatch(connect());

  let color;
  switch (status) {
    case ConnectionStatus.Connected:
      color = 'success';
      break;
    case ConnectionStatus.Connecting:
      color = 'warning';
      break;
    case ConnectionStatus.Disconnected:
      color = 'danger';
      break;
  }

  return (
    <Container>
      <Row className="align-items-center">
        <Col>
          <Alert variant={color}>{statusText}</Alert>
        </Col>
        <Col md='auto'>
          <Button disabled={status !== ConnectionStatus.Disconnected} variant="primary" onClick={retryConnection}>Connect</Button>
        </Col>
      </Row>
    </Container>
  )
}
