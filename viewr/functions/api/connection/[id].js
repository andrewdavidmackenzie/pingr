// Return a JSON for the Value of the entry in the ConnectionDeviceStatus KV Namespace
// for the entry with key == 'id'
export async function onRequestGet(context) {
  const connection_device_id = context.params.id;

  if (!connection_device_id) {
    return new Response('Not found', { status: 404 })
  }

  const state_change = await context.env.CONNECTION_DEVICE_STATUS.get(connection_device_id);

  if (!state_change) {
    return new Response('Not found', { status: 404 })
  }

  return new Response(state_change);
}