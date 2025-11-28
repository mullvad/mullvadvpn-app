import { useCallback, useRef } from 'react';
import styled from 'styled-components';

import { useAppContext } from '../../../context';
import { Button } from '../../../lib/components';
import { Image } from '../../../lib/components/image/index';
import { TextField, useTextField } from '../../../lib/components/text-field';
import { useBoolean } from '../../../lib/utility-hooks';
import { useSelector } from '../../../redux/store';
import { ModalAlert, ModalMessage } from '../../Modal';

const Container = styled.div({
  display: 'flex',
  backgroundColor: 'rgba(255, 255, 255, 0.5)',
  borderRadius: '50%',
  justifyContent: 'center',
  alignItems: 'center',

  width: '40px',
  height: '40px',

  '&&:hover': {
    backgroundColor: 'rgba(255, 255, 255, 0.7)',
  },
});

const BigIconContainer = styled(Container)({
  alignSelf: 'center',
  width: '100px',
  height: '100px',
});

const StyledTextFieldInput = styled(TextField.Input)({
  width: '93%',
  margin: '1px',
});

export function AppMainHeaderRouterButton() {
  const { connectToRouter } = useAppContext();
  const [modalVisible, openModal, closeModal] = useBoolean();

  const currentRouterIp = useSelector((state) => state.userInterface.currentRouterIp);

  const inputRef = useRef<HTMLInputElement>(null);
  const { value, handleChange } = useTextField({ inputRef, defaultValue: currentRouterIp });

  const connect = useCallback(() => {
    connectToRouter(value);
  }, [value, connectToRouter]);

  return (
    <>
      <Container onClick={openModal}>
        <Image source="router" height={20} />
      </Container>

      <ModalAlert
        isOpen={modalVisible}
        buttons={[
          <Button key="back" onClick={closeModal}>
            <Button.Text>{'Cancel'}</Button.Text>
          </Button>,
          <Button key="back" onClick={connect}>
            <Button.Text>{'Connect!'}</Button.Text>
          </Button>,
        ]}
        close={closeModal}>
        <BigIconContainer>
          <Image source="router" height={70} />
        </BigIconContainer>
        <ModalMessage>Enter router address:</ModalMessage>
        <TextField>
          <StyledTextFieldInput ref={inputRef} value={value} onChange={handleChange} />
        </TextField>
      </ModalAlert>
    </>
  );
}
