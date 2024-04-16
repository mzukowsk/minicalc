import { useState } from 'preact/hooks';
import Button from 'react-bootstrap/Button';
import Modal from 'react-bootstrap/Modal';
import { useAppDispatch, useAppSelector } from './store';
import { useShowModal } from './modal-slice';

export function ModalHost() {
  const modal = useAppSelector((state) => state.modal.modalData);
  const dispatch = useAppDispatch();
  const showModal = useShowModal(dispatch);

  if (modal) {
    const [show, setShow] = useState(true);

    const buttons = modal.buttons || [{ variant: "primary", label: "OK" }];

    const closeModal = (callback?: () => void) => {
      setShow(false);
      if (callback)
        callback();
    };

    const modalExited = () => {
      showModal();
      setShow(true);
    };

    return (
      <Modal show={show} onExited={() => modalExited()}>
        <Modal.Header>
          <Modal.Title>{modal.title}</Modal.Title>
        </Modal.Header>
        <Modal.Body>{modal.body}</Modal.Body>
        <Modal.Footer>
          {buttons.map(b => <Button variant={b.variant} onClick={() => closeModal(b.callback)}>{b.label}</Button>)}
        </Modal.Footer>
      </Modal>
    );
  } else {
    return null;
  }  
}
