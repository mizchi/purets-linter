// Error: throw and delete are not allowed

function process(data: any) {
  if (!data) {
    throw new Error("Invalid data");
  }
  
  delete data.prop;
  return data;
}

export default process;