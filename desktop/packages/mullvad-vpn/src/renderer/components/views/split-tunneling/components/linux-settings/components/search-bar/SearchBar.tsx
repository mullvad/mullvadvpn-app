import { StyledSearchBar } from '../../../../SplitTunnelingStyles';
import { useLinuxSettingsContext } from '../../LinuxSettingsContext';

export function SearchBar() {
  const { searchTerm, setSearchTerm } = useLinuxSettingsContext();

  return <StyledSearchBar searchTerm={searchTerm} onSearch={setSearchTerm} />;
}
