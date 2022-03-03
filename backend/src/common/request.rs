use std::marker::PhantomData;

#[derive(Debug)]
pub struct Error(pub String);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ID(pub u128);

pub trait Get<T, U> {
    fn get(&self) -> Result<U, Error>;
}

pub trait Set<T, U> {
    fn set(&mut self, target: &U) -> Result<(), Error>;
}

pub trait GetRequest<T, U, R>
where
    T: Get<T, U>,
    R: GetResponse<T, U>,
{
    fn get_response(self, target: &T) -> R;
    fn get_id(&self) -> ID;
}

pub trait SetRequest<T, U, R>
where
    T: Set<T, U>,
    R: SetResponse<T, U>,
{
    fn get_candidate(&self) -> &U;
    fn get_response(self, target: &mut T) -> R;
    fn get_id(&self) -> ID;
}

pub trait GetResponse<T, U>
where
    T: Get<T, U>,
{
    fn get_result(self) -> Result<U, Error>;
    fn get_id(&self) -> ID;
}

pub trait SetResponse<T, U>
where
    T: Set<T, U>,
{
    fn get_candidate(&self) -> &U;
    fn get_result(self) -> Result<(), Error>;
    fn get_id(&self) -> ID;
}

pub struct BasicGetResponse<T, U>(pub ID, pub Result<U, Error>, pub PhantomData<T>);

impl<T, U> GetResponse<T, U> for BasicGetResponse<T, U>
where
    T: Get<T, U>,
{
    fn get_id(&self) -> ID {
        return self.0;
    }
    fn get_result(self) -> Result<U, Error> {
        return self.1;
    }
}

pub struct BasicGetRequest<T, U>(pub ID, pub PhantomData<T>, pub PhantomData<U>);

impl<T, U> GetRequest<T, U, BasicGetResponse<T, U>> for BasicGetRequest<T, U>
where
    T: Get<T, U>,
{
    fn get_response(self, target: &T) -> BasicGetResponse<T, U> {
        let result = target.get();
        BasicGetResponse(self.0, result, PhantomData)
    }
    fn get_id(&self) -> ID {
        return self.0;
    }
}

pub struct BasicSetResponse<T, U>(pub ID, pub U, pub Result<(), Error>, pub PhantomData<T>);

impl<T, U> SetResponse<T, U> for BasicSetResponse<T, U>
where
    T: Set<T, U>,
{
    fn get_id(&self) -> ID {
        return self.0;
    }
    fn get_result(self) -> Result<(), Error> {
        return self.2;
    }
    fn get_candidate(&self) -> &U {
        return &self.1;
    }
}

pub struct BasicSetRequest<T, U>(pub ID, pub U, pub PhantomData<T>);

impl<T, U> SetRequest<T, U, BasicSetResponse<T, U>> for BasicSetRequest<T, U>
where
    T: Set<T, U>,
{
    fn get_response(self, target: &mut T) -> BasicSetResponse<T, U> {
        let result = target.set(&self.1);
        BasicSetResponse(self.0, self.1, result, self.2)
    }
    fn get_id(&self) -> ID {
        return self.0;
    }
    fn get_candidate(&self) -> &U {
        return &self.1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Data1(i32);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Data2(i32);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Simple(Data1, Data2);

    impl Get<Simple, Data1> for Simple {
        fn get(&self) -> Result<Data1, Error> {
            return Ok(self.0);
        }
    }

    impl Get<Simple, Data2> for Simple {
        fn get(&self) -> Result<Data2, Error> {
            return Ok(self.1);
        }
    }

    impl Set<Simple, Data1> for Simple {
        fn set(&mut self, target: &Data1) -> Result<(), Error> {
            self.0 = *target;
            return Ok(())
        }
    }

    impl Set<Simple, Data2> for Simple {
        fn set(&mut self, target: &Data2) -> Result<(), Error> {
            self.1 = *target;
            return Ok(())
        }
    }

    #[test]
    fn test_get() {
        let data = Simple(Data1(32), Data2(42));
        assert_eq!(Get::<Simple, Data1>::get(&data).unwrap(), data.0);
        assert_eq!(Get::<Simple, Data2>::get(&data).unwrap(), data.1);
    }

    #[test]
    fn test_get_request() {
        let request = BasicGetRequest::<Simple, Data1>(ID(0u128), PhantomData, PhantomData);
        assert_eq!(request.get_id(), ID(0u128));
    }

    #[test]
    fn test_get_response() {
        let data = Simple(Data1(32), Data2(42));
        let request = BasicGetRequest::<Simple, Data1>(ID(0u128), PhantomData, PhantomData);
        let response = request.get_response(&data);
        assert_eq!(response.get_id(), ID(0u128));
        assert_eq!(response.get_result().unwrap(), Data1(32));
    }

    #[test]
    fn test_set() {
        let mut data = Simple(Data1(32), Data2(42));
        assert_eq!(Set::<Simple, Data1>::set(&mut data, &Data1(50)).unwrap(), ());
        assert_eq!(Set::<Simple, Data2>::set(&mut data, &Data2(60)).unwrap(), ());
        assert_eq!(Get::<Simple, Data1>::get(&data).unwrap(), Data1(50));
        assert_eq!(Get::<Simple, Data2>::get(&data).unwrap(), Data2(60));
    }

    #[test]
    fn test_set_request() {
        let request = BasicSetRequest::<Simple, Data1>(ID(0u128), Data1(53), PhantomData);
        assert_eq!(request.get_id(), ID(0u128));
        assert_eq!(*request.get_candidate(), Data1(53));
    }

    #[test]
    fn test_set_response() {
        let mut data = Simple(Data1(32), Data2(42));
        let request = BasicSetRequest::<Simple, Data1>(ID(0u128), Data1(53), PhantomData);
        let response = request.get_response(&mut data);
        assert_eq!(response.get_id(), ID(0u128));
        assert_eq!(response.get_candidate(), &Data1(53));
        assert_eq!(response.get_result().unwrap(), ());
        assert_eq!(data.0, Data1(53));
    }
}
