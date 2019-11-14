import React, { Component } from 'react';
import {
  Card,
  CardBody,
  CardHeader,
  Col,
  Row
} from 'reactstrap';
import System from './System';

class SystemList extends Component {
  onUpdate = () => {
    this.setState({});
  }

  componentDidMount() {
    this.props.systems.on('add', this.onUpdate);
    this.props.systems.on('remove', this.onUpdate);
  }

  componentWillUnmount() {
    this.props.systems.off('add', this.onUpdate);
    this.props.systems.off('remove', this.onUpdate);
  }

  render() {
    // ResClient Collections are iterables, but not arrays.
    return (
     
              <Row>

                {this.props.systems && Array.from(this.props.systems).map(system =>
                  <Col xs="3" sm="3" lg="3"><System key={system.name} systems={this.props.systems} system={system} /></Col>
                )}

              </Row>
     
    );
  }
}

export default SystemList;