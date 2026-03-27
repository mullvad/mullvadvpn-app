export function searchMatchesLocation(location: string, searchTerm: string): boolean {
  const lowerCaseSearchTerm = searchTerm.toLocaleLowerCase();
  const lowerCaseLocation = location.toLocaleLowerCase();

  const matchesLocation = lowerCaseLocation.includes(lowerCaseSearchTerm);

  return matchesLocation;
}
