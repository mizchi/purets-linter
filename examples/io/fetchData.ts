/**
 * @allow net
 * Fetches data from an API endpoint.
 */
export async function fetchData(url: string): Promise<any> {
  const response = await fetch(url);
  return response.json();
}