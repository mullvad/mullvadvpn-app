import React from 'react';

import {
  compareRelayLocationGeographical,
  type CustomListError,
  type RelayLocationGeographical,
} from '../../../../shared/daemon-rpc-types';
import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useCustomLists() {
  const customLists = useSelector((state) => state.settings.customLists);
  const {
    createCustomList: contextCreateCustomList,
    updateCustomList,
    deleteCustomList,
  } = useAppContext();

  const createCustomList = React.useCallback(
    async (name: string): Promise<void | CustomListError> => {
      return contextCreateCustomList({
        name,
        locations: [],
      });
    },
    [contextCreateCustomList],
  );

  const addLocationToCustomList = React.useCallback(
    async (listId: string, location: RelayLocationGeographical) => {
      const list = customLists.find((list) => list.id === listId);
      if (list === undefined) {
        log.error(`Custom list with id ${listId} not found`);
        return;
      }
      const updatedList = {
        ...list,
        locations: [...list.locations, location],
      };

      try {
        await updateCustomList(updatedList);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to edit custom list ${list.id}: ${error.message}`);
      }
    },
    [customLists, updateCustomList],
  );

  const removeLocationFromCustomList = React.useCallback(
    async (listId: string, location: RelayLocationGeographical) => {
      const list = customLists.find((list) => list.id === listId);
      if (list === undefined) {
        log.error(`Custom list with id ${listId} not found`);
        return;
      }
      const updatedList = {
        ...list,
        locations: list.locations.filter(
          (listLocation) => !compareRelayLocationGeographical(listLocation, location),
        ),
      };

      try {
        await updateCustomList(updatedList);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to edit custom list ${list.id}: ${error.message}`);
      }
    },
    [customLists, updateCustomList],
  );

  const updateCustomListName = React.useCallback(
    async (listId: string, name: string) => {
      const list = customLists.find((list) => list.id === listId);
      if (list === undefined) {
        log.error(`Custom list with id ${listId} not found`);
        return;
      }
      const updatedList = { ...list, name };
      try {
        return await updateCustomList(updatedList);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update list:', error.message);
      }
    },

    [customLists, updateCustomList],
  );

  return {
    customLists,
    createCustomList,
    addLocationToCustomList,
    removeLocationFromCustomList,
    updateCustomListName,
    deleteCustomList,
  };
}
