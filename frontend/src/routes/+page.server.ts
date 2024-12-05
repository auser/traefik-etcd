
export async function load({ url }) {
  const configId = url.searchParams.get('configId');

  return {
    configId
  };

}

