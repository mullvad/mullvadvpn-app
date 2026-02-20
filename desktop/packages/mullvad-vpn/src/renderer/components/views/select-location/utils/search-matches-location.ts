export function searchMatchesLocation(location: string, searchTerm: string): boolean {
  const lowerCaseSearchTerm = searchTerm.toLowerCase();
  const lowerCaseLocation = location.toLowerCase();
  if (lowerCaseLocation.includes(lowerCaseSearchTerm)) {
    return true;
  }
  return false;
}
