export function shouldLocationSelectorExpand(scrollTop: number): boolean {
  if (scrollTop > 30) {
    return false;
  }

  return true;
}
