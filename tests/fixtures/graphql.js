import gql from "graphql-tag";

const query = gql`
  query Me {
    me { id }
  }
`;

graphql("https://api.example.com/graphql", query);
