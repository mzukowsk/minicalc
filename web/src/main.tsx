import { Provider } from 'react-redux';
import { render } from 'preact'
import { ModalHost } from './modal-host';
import { App } from './app'
import { store } from './store';
import 'bootstrap/dist/css/bootstrap.min.css';
import './index.css'

function AppContext() {
  return (
      <Provider store={store}>
        <App />
        <ModalHost />
      </Provider>
  )
}

render(<AppContext />, document.getElementById('app')!)
