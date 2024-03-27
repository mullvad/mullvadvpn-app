import { useCallback, useEffect, useRef } from 'react';
import { useParams } from 'react-router';
import { sprintf } from 'sprintf-js';

import {
  CustomProxy,
  NamedCustomProxy,
  NewAccessMethodSetting,
} from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useScheduler } from '../../shared/scheduler';
import { useAppContext } from '../context';
import { useApiAccessMethodTest } from '../lib/api-access-methods';
import { useHistory } from '../lib/history';
import { useSelector } from '../redux/store';
import { SettingsForm } from './cell/SettingsForm';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType } from './Modal';
import { NavigationBar, NavigationContainer, NavigationItems, TitleBarItem } from './NavigationBar';
import { NamedProxyForm } from './ProxyForm';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';
import { StyledContent, StyledNavigationScrollbars, StyledSettingsContent } from './SettingsStyles';
import { SmallButton } from './SmallButton';

export function EditApiAccessMethod() {
  return (
    <SettingsForm>
      <AccessMethodForm></AccessMethodForm>
    </SettingsForm>
  );
}

function AccessMethodForm() {
  const history = useHistory();
  const { addApiAccessMethod, updateApiAccessMethod } = useAppContext();
  const methods = useSelector((state) => state.settings.apiAccessMethods.custom);

  const [testing, testResult, testApiAccessMethod, resetTestResult] = useApiAccessMethodTest(
    false,
    500,
  );
  const saveScheduler = useScheduler();

  // Use id in url to figure out which method is to be edited. undefined means this is a new method.
  const { id } = useParams<{ id: string | undefined }>();
  // Ugly way of iterating over all access methods, but it works.
  const method = methods.find((method) => method.id === id);

  const updatedMethod = useRef<NewAccessMethodSetting<CustomProxy> | undefined>(method);

  const save = useCallback(() => {
    if (updatedMethod.current !== undefined) {
      resetTestResult();
      if (id === undefined) {
        void addApiAccessMethod(updatedMethod.current);
      } else {
        void updateApiAccessMethod({ ...updatedMethod.current, id });
      }
      history.pop();
    }
  }, [updatedMethod.current, id]);

  const onSave = useCallback(
    async (newMethod: NamedCustomProxy) => {
      const enabled = id === undefined ? true : method?.enabled ?? true;
      updatedMethod.current = { ...newMethod, enabled };
      if (
        updatedMethod.current !== undefined &&
        (await testApiAccessMethod(updatedMethod.current as CustomProxy))
      ) {
        // Hide the save dialog after 1.5 seconds.
        saveScheduler.schedule(save, 1500);
      }
    },
    [updatedMethod, save, history.pop],
  );

  const title = getTitle(id === undefined);
  const subtitle = getSubtitle(id === undefined);

  return (
    <BackAction action={history.pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <TitleBarItem>{title}</TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <StyledNavigationScrollbars fillContainer>
              <StyledContent>
                <SettingsHeader>
                  <HeaderTitle>{title}</HeaderTitle>
                  <HeaderSubTitle>{subtitle}</HeaderSubTitle>
                </SettingsHeader>

                <StyledSettingsContent>
                  {id !== undefined && method === undefined ? (
                    <span>Failed to open method</span>
                  ) : (
                    <NamedProxyForm proxy={method} onSave={onSave} onCancel={history.pop} />
                  )}
                </StyledSettingsContent>

                <TestingDialog
                  name={updatedMethod.current?.name ?? ''}
                  newMethod={id === undefined}
                  testing={testing}
                  testResult={testResult}
                  cancel={resetTestResult}
                  save={save}
                />
              </StyledContent>
            </StyledNavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function getTitle(isNewMethod: boolean) {
  return isNewMethod
    ? messages.pgettext('api-access-methods-view', 'Add method')
    : messages.pgettext('api-access-methods-view', 'Edit method');
}

function getSubtitle(isNewMethod: boolean) {
  return isNewMethod
    ? messages.pgettext('api-access-methods-view', 'Adding a new API access method also tests it.')
    : messages.pgettext('api-access-methods-view', 'Editing an API access method also tests it.');
}

interface TestingDialogProps {
  name: string;
  newMethod: boolean;
  testing: boolean;
  testResult?: boolean;
  cancel: () => void;
  save: () => void;
}

function TestingDialog(props: TestingDialogProps) {
  const type = props.testing
    ? ModalAlertType.loading
    : props.testResult
    ? ModalAlertType.success
    : ModalAlertType.failure;
  const prevType = useRef<ModalAlertType>(type);

  const isOpen = props.testing || props.testResult !== undefined;
  const typeValue = isOpen ? type : prevType.current;

  useEffect(() => {
    if (isOpen) {
      prevType.current = type;
    }
  }, [type]);

  return (
    <ModalAlert
      isOpen={isOpen}
      type={typeValue}
      gridButtons={getTestingDialogButtons(typeValue, props.save, props.cancel)}
      close={props.cancel}
      title={getTestingDialogTitle(typeValue, props.newMethod)}
      message={getTestingDialogSubTitle(typeValue, props.newMethod, props.name)}
    />
  );
}

function getTestingDialogTitle(type: ModalAlertType, newMethod: boolean) {
  switch (type) {
    case ModalAlertType.success:
      return newMethod
        ? messages.pgettext('api-access-methods-view', 'API reachable, adding method…')
        : messages.pgettext('api-access-methods-view', 'API reachable, saving method…');
    case ModalAlertType.failure:
      return newMethod
        ? messages.pgettext('api-access-methods-view', 'API unreachable, add anyway?')
        : messages.pgettext('api-access-methods-view', 'API unreachable, save anyway?');
    default:
    case ModalAlertType.loading:
      return messages.pgettext('api-access-methods-view', 'Testing method...');
  }
}

function getTestingDialogSubTitle(type: ModalAlertType, newMethod: boolean, name: string) {
  switch (type) {
    case ModalAlertType.failure:
      return newMethod
        ? sprintf(
            messages.pgettext(
              'api-access-methods-view',
              'The API could not be reached using the %(name)s method.',
            ),
            { name },
          )
        : messages.pgettext(
            'api-access-methods-view',
            'Clicking “Save” changes the in use method.',
          );
    default:
      return undefined;
  }
}

function getTestingDialogButtons(type: ModalAlertType, save: () => void, cancel: () => void) {
  const saveButton = (
    <SmallButton key="confirm" onClick={save}>
      {messages.gettext('Save')}
    </SmallButton>
  );
  const cancelButton = (
    <SmallButton key="cancel" onClick={cancel}>
      {messages.gettext('Cancel')}
    </SmallButton>
  );
  const disabledCancelButton = (
    <SmallButton key="cancel" onClick={cancel} disabled>
      {messages.gettext('Cancel')}
    </SmallButton>
  );

  switch (type) {
    case ModalAlertType.success:
      return [disabledCancelButton];
    case ModalAlertType.failure:
      return [cancelButton, saveButton];
    case ModalAlertType.loading:
    default:
      return [cancelButton];
  }
}
