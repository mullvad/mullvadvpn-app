export function searchMatchesLocation(location: string, searchTerm: string): boolean {
  const lowerCaseSearchTerm = searchTerm.toLowerCase();
  const lowerCaseLocation = location.toLowerCase();

  const matchesLocation = lowerCaseLocation.includes(lowerCaseSearchTerm);

  return matchesLocation;
}
